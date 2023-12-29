/*!
后台服务
- 负责通过UDP接收行情信息
- 并进行加工，压缩数据（时间合并），各种TOP序列的生成。
- 绘制通知 ctx.request_repaint();
- 其他的策略，算法执行。
 
接收UDP数据依赖foxy,9个字段，顺序定义为：
    1. AveragePrice,
    2. buy,
    3. sell,
    4. InstrumentID,
    5. LastPrice,
    6. OpenPrice,
    7. TradingDay,
    8. UpdateTime,
    9. Volume;

*/

use crate::*;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::net::UdpSocket as TokioUdpSocket;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
/**
FOXY数据格式的定义
*/
const FOXY_TICK_LEN: usize = 9;
const FOXY_AVERAGE: usize = 0;
const FOXY_BUY: usize = 1;
const FOXY_SELL: usize = 2;
const FOXY_INSTRUMENTID: usize = 3;
const FOXY_LASTPRICE: usize = 4;
//const FOXY_OPENPRICE: usize = 5;
const FOXY_TRADINGDAY: usize = 6;
const FOXY_UPDATETIME: usize = 7;
const FOXY_VOLUME: usize = 8;

#[derive(Deserialize, Default, Debug, Serialize, Clone)]
struct Notify {   
    #[serde(default)]    
    pub recv_id : String,        
    #[serde(default)]
    pub recv_type : String,
    #[serde(default)]
    pub content : String,
    #[serde(default)]
    pub msg_type :String,
}
struct NotifyHandle{
    pub notify_his: Vec<(i32, DateTime<Local>, String)>,
}

impl NotifyHandle{
    pub fn new() -> Self{
        Self{
            notify_his: vec![]
        }
    }
    pub async fn add_notify(&mut self, first_av:f32, tick: &SimpleTick, warn : f32 , fname :&str ){
        let val = (tick.last_price - tick.average_price).abs();
        
        let min_v = 0.2* warn;
        let small_half = 0.8f32; 

        let updown = if tick.average_price > first_av + min_v    {
            "大势上涨".to_string()
        } else if tick.average_price < first_av - min_v   {
            "大势下跌".to_string()
        }else { "大势震荡".to_string() };

        // 对于单边趋势，小边方向的振幅 0.6 即是机会
        let new_warn = if tick.average_price > first_av + min_v && tick.last_price < tick.average_price {  warn*small_half }
            else if tick.average_price < first_av - min_v && tick.last_price > tick.average_price  {  warn*small_half }
            else {warn };

        if val >= new_warn {
            let updown = if tick.last_price > tick.average_price {
                format!("{}, 上行预警", updown)
            } else {
                format!("{}, 下行预警", updown)
            };
            
            let level = (val*10.0/new_warn) as i32 - 9;
            let now = Local::now();
            
            if self.notify_his.len() == 0 {
                let msg = format!("{}, {}-{}, 最新价:{:.2} (av={:.2})",fname, updown, level, tick.last_price, tick.average_price );
                notify(&msg).await.unwrap();
                self.notify_his.push((level, now, msg));
            }else{
                if  self.notify_his[ self.notify_his.len()-1].0 > level && now -  self.notify_his[ self.notify_his.len()-1].1 > chrono::Duration::seconds(60) {
                    let msg = format!("{}, {}:{}, {:.2} (av = {:.2})", fname, updown, level, tick.last_price, tick.average_price );
                    if  self.notify_his[ self.notify_his.len()-1].0 > level {
                        notify(&msg).await.unwrap();
                    }
                    self.notify_his.push((level, now, msg));
                }
            }            
            while  self.notify_his.len() > 10 {
                self.notify_his.remove(0);
            }
        } 
    }
}
/**
后台的异步接收行情线程，负责接收UPD发来的行情信息，并进行加工处理和显示通知。
- 借助Arc<Shared>实现与egui线程的数据共享
- 借助egui::Context句柄，实现界面绘制通知。
- 通过broadcast实现优雅关机退出。
*/
pub(crate) struct Worker {
    pub shared: Arc<Shared>,
   // pub shutdown: broadcast::Receiver<()>,
    pub cmd_chan: broadcast::Receiver< String >,
    pub ctx: egui::Context,
}

impl Worker {
    pub fn new(shared: Arc<Shared>,/* shutdown: broadcast::Receiver<()>, */ cmd_chan: broadcast::Receiver<String>,  ctx: egui::Context) -> Self {
        Self {
            shared,
           // shutdown,
            cmd_chan,
            ctx,
        }
    }
/**
后台的异步执行线程。
- 可以借助ticker来实现定时策略分析任务。

 */
    pub async fn run(&mut self) -> Result<()> {
        //let mut ticker = tokio::time::interval(Duration::from_secs(3));
        //ticker.tick().await; // 首次立刻执行，不合适。
        // let addr = format!("0.0.0.0:{}", 12346);
        let mut futures_name = self.shared.get_futures_name();
        let socket = TokioUdpSocket::bind(&self.shared.udp_addr).await.unwrap(); //  "192.168.2.110:12345")?;
        let mut buf = [0; 1024];
        let mut counter = 0;        
        let mut trade_day = Local::now().format("%Y%m%d").to_string();  // 交易日,收到不同交易日时，保存，清空！！
        // let mut notify_his: Vec<(i32, DateTime<Local>, String)> = vec![];
        let mut first_av = 0.0f32;
        let mut notify =  NotifyHandle::new();
        loop {
            tokio::select! {
                     ret = socket.recv(&mut buf) => {
                        if let Ok(amt) = ret {
                             if amt == 0 {
                                 error!("received 0 bytes");
                                 continue;
                             }
                             let src = String::from_utf8(buf[..amt].to_vec())?;
                             let vs: Vec<&str> = src.split(';').collect();
                             //
                             if vs.len() != FOXY_TICK_LEN {
                                 error!("invalid message");
                                 continue;
                             }

                             let mut tick = SimpleTick {
                                average_price: vs[FOXY_AVERAGE ].parse::<f32>()?,
                                buy: vs[FOXY_BUY].parse::<i32>()?,
                                sell: vs[FOXY_SELL].parse::<i32>()?,
                                last_price: vs[FOXY_LASTPRICE].parse::<f32>()?,
                                update_time: vs[FOXY_UPDATETIME][0..7].to_string(), // 09:20:10
                                volume: vs[FOXY_VOLUME].parse::<i32>()?
                             };            
                                
                             // 某些商品，如塑料l2401,averaged_price是一手（5吨）的均价，需要除以参数5.0，计算出每吨的均价。                             
                            if self.shared.config.average_package > 1 {
                                tick.average_price = tick.average_price / self.shared.config.average_package as f32;
                            }
                            // 如果交易日变更, 收到不同交易日时，则自动保存，清空，开始新的交易日数据！！
                            if trade_day != vs[FOXY_TRADINGDAY] {
                                error!("{} is not the same trading day as {}, save and clear now!", trade_day, vs[FOXY_TRADINGDAY]);                                
                                self.shared.write_cvs(&futures_name).unwrap_or_default();
                                self.shared.clear();
                                futures_name = vs[FOXY_INSTRUMENTID].to_owned();
                                self.shared.set_futures_name(&futures_name);
                                first_av = tick.average_price;
                                trade_day = vs[FOXY_TRADINGDAY].to_string();
                            }             

                            if futures_name.len() == 0 { // 这个逻辑应该永远不会执行吧？？ 不是，Delete key后就可能触发！！
                                futures_name = vs[FOXY_INSTRUMENTID].to_owned();                            
                                self.shared.set_futures_name(&futures_name);       
                                first_av = tick.average_price;                        
                            }

                            if futures_name == vs[FOXY_INSTRUMENTID]  {             
                                if first_av == 0.0 {
                                    first_av = tick.average_price;
                                }
                                notify.add_notify(first_av, &tick, self.shared.config.warn_value, &futures_name).await;
                                if self.shared.add_tick(tick) {     
                                    self.shared.strategy();                             
                                    self.ctx.request_repaint();
                                }
                            }else{
                                if counter == 0 {
                                    error!("data's futures_code is {}, But received is {}", &futures_name, &vs[FOXY_INSTRUMENTID]);
                                }
                                counter = (counter + 1)%200;
                            }
                        }
                     },
                     /*
                 _ = ticker.tick() => {
                     counts += 1;
                     println!("tick: {}", counts);

                 }, 
                 _ = self.shutdown.recv() => {  // 接收到退出信号
                     self.shared.write_cvs(&futures_name).unwrap_or_default();                   
                     error!("received shutdown, save data and exit");
                     self.shared.report();
                     break;
                 }, */
                 cmd = self.cmd_chan.recv() => {  // 接收命令
                    match cmd {
                        Ok(cmd) => {
                            if cmd == "clear" {
                                self.shared.clear();
                                futures_name.clear();
                                first_av = 0.0;
                            } 
                            else if cmd == "exit" || cmd == "quit" {
                                self.shared.write_cvs(&futures_name).unwrap_or_default();
                                error!("received shutdown, save data and gracefull to EXIT!");
                                self.shared.report();
                                break;
                            }
                            else {
                                println!("hello, {}", cmd);
                            }
                        },
                        _ => {}
                    }                  
                },
            }
        }
        Ok(())
    }
   

}

/**
method = 'https://www.mark.lengsh.cn/xman/sendmessage'
headers = {'Content-Type': 'application/json', "Authorization":"Bearer xman"
payload = {'recv_id':recv_id, 'recv_type':recv_type, 'content': content,'msg_type': msg_type}
r = requests.post(method,  data =json.dumps(payload), headers=headers)                                                                                                                                  

*/    
async fn notify(msg: &str) -> Result<()> {
    let open_id = "ou_67522248a19f98afac16f0f64c1d34b8";
    let client = reqwest::Client::new();
    let svr = "https://www.mark.lengsh.cn/xman/sendmessage";
    let mut headers = HeaderMap::new();    
    headers.insert("Authorization", "Bearer xman".parse()?);
    headers.insert("Content-Type", "application/json; charset=utf-8".parse()?);
   
    let notify = Notify {
        recv_id: open_id.to_string(),
        recv_type: "open_id".to_string(),
        msg_type: "text".to_string(),
        content: msg.to_string()
    };
    
    let r  = client
        .post(svr)
        .headers(headers)
        .json(&notify)
        .send()
        .await?
        .text()
        .await?;
    debug!("{:#?}", r);   
    
    return Ok(());
}


#[test]
fn test_notify() {
    let re = notify("西红柿鸡蛋汤怎么做？" );
    let rt = tokio::runtime::Runtime::new().unwrap();
    let r = rt.block_on(re);
    println!("{:#?}", r);
}
