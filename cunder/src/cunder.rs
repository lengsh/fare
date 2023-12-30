/*! 期货商品检测处理
 主要包括：
 1. 行情数据加工
 2. 绘制图表
 3. 绘制警告标志
 4. 后台打印警示信息
 5. 数据csv保存

 操作方式：
 - 鼠标左键双击，增加标记，鼠标右键双击标记，将删除当前标记；
 - DELETE删除当前所有数据。
 - F1，打印配置信息
 - F2，打印当前运行数据的样本（部分）
 - F3，打印当前各商品申请交易TOP10样本点（tick特殊变量定义：buy表示申卖申买总和，sign表示全量数据中的index）
 - F4, 重新加载配置文件信息（只有期货信息有效，窗口参数和端口参数无效）
 - F5，保存当前数据到csv文件

提示标志：
 - 小红点：高速变价点(价格变动速度超标)；
 - 小绿点：人工标注点(鼠标左键双击添加，右键双击删除)；
 - 小蓝点：推荐下单点(变价幅度+流量热度)；
 - 小红柱：流量热度位(申请交易量爆表)；
 - 大绿叉：单边危险商品，不建议交易；
 - 大红点：涨势超标商品；
 - 大绿点：跌势超标商品；
 - 酱紫线：流量热度TOP5(申请交易量TOP5);
*/
use ggez::{
    glam::*,
    graphics::{self, Color, DrawMode, FillOptions},
    input::{keyboard, keyboard::KeyInput, mouse::MouseButton},
    Context, GameResult,
};

use crate::*;
use anyhow::{bail, Ok, Result};
use chrono::prelude::*;
use csv::{Reader, ReaderBuilder, Writer, WriterBuilder};
use std::collections::HashMap;
use std::{net::UdpSocket, path::Path, path::PathBuf, time::Duration};
use tracing::{debug, error, info};

use std::sync::mpsc;
/**
鼠标点击记录，用于做有效的双击判断
*/
struct Click {
    x: f32,
    y: f32,
    time: DateTime<Utc>,
    button: MouseButton,
}
/**
实现ggez trait的图像绘制对象
*/
pub struct Cunder {
    // 画图区域宽
    width: f32,
    // 画图区域高
    height: f32,
    meshes: Vec<graphics::Mesh>,

    // 监听的udp端口，接收行情数据
    socket: UdpSocket,
    // 系统配置
    cfg: Config,
    futures: HashMap<String, Vec<SimpleTick>>,
    // 各区域futures Name
    tests: Vec<(String, f32, f32, Color)>,
    // 鼠标点击记录
    last_click: Click,
    futures_index: Vec<String>,
    // futures_timer: HashMap<String, DateTime<Utc>>,
    //work path
    work_path: String,
    rebuild_counter: i32,
    tx: mpsc::Sender<(String, usize, String)>,
    bs_tops: HashMap<String, Vec<SimpleTick>>,
    cfg_file: String,
}

impl Cunder {
    /// Load images and create meshes.
    pub fn new(
        _ctx: &mut Context,
        cfg: Config,
        cfg_file: String,
        work_path: String,
        height: f32,
        width: f32,
        tx: mpsc::Sender<(String, usize, String)>,
    ) -> GameResult<Cunder> {
        let addr = format!("0.0.0.0:{}", cfg.udp_port);
        let socket = UdpSocket::bind(&addr)?; //  "192.168.2.110:12345")?;
        socket.set_read_timeout(Some(Duration::new(0, 1000)))?;
        let s = Cunder {
            cfg_file,
            width,
            height,
            work_path,
            meshes: vec![],
            futures: HashMap::new(),
            futures_index: vec![],
            //      futures_timer: HashMap::new(),
            socket,
            cfg,
            rebuild_counter: 0,
            tests: vec![],
            last_click: Click {
                x: 0.0,
                y: 0.0,
                time: Utc::now(),
                button: MouseButton::Other(0),
            },
            tx,
            bs_tops: HashMap::new(),
        };
        GameResult::Ok(s)
    }
}

impl Cunder {
    fn is_rebuild(&mut self) -> bool {
        self.rebuild_counter = (self.rebuild_counter + 1) % 10;
        self.rebuild_counter == 1
    }
    fn build_meshes(&mut self, ctx: &mut Context, force_build: bool) -> Result<()> {
        let count = self.futures.len();
        if count == 0 {
            return Ok(());
        }

        let rebuild_index = if self.futures_index.len() != count {
            self.futures_index.clear();
            true
        } else {
            false
        };

        let mesh_build = if force_build || self.is_rebuild() {
            true
        } else {
            false
        };
        //error!("build_meshes : {}", mesh_build);
        let fheight = self.height / ((1 + (count - 1) / 2) as f32);
        let one_left = BORDER_WIDTH;
        let two_left = self.width / 2.0 + BORDER_WIDTH;
        let one_width = self.width / 2.0 - 2.0 * BORDER_WIDTH;

        let mesh = &mut graphics::MeshBuilder::new();
        if mesh_build {
            self.tests.clear();
            self.meshes.clear();
        }

    
        for (i, (fname, ticks)) in self.futures.iter_mut().enumerate() {
            if rebuild_index {
                self.futures_index.push(fname.to_string());
            }

            if ticks.len() == 0 {
                continue;
            }

            let mut max_buy = (0,0);
            let mut max_sell = (0,0);

            let mut max_v = ticks[0].last_price;
            let mut min_v = ticks[0].last_price;

            let mut aor = Aor::new(fname.to_string());

            for idx in 0..ticks.len() {
                // 查找最大的Buy和Sell, 用于后续的图形提醒！！
                if ticks[idx].buy > max_buy.1 {
                    max_buy = (idx, ticks[idx].buy);
                }
                if ticks[idx].sell > max_sell.1 {
                    max_sell = (idx, ticks[idx].sell);
                }
                // 获取价格最大，最小值，用于计算价格图形比例
                if ticks[idx].last_price > max_v {
                    max_v = ticks[idx].last_price;
                }
                if ticks[idx].last_price < min_v {
                    min_v = ticks[idx].last_price;
                }
                if ticks[idx].average_price > max_v {
                    max_v = ticks[idx].average_price;
                }
                if ticks[idx].average_price < min_v {
                    min_v = ticks[idx].average_price;
                }
                aor.next(&ticks[idx])?;
            }
            if (max_v - min_v) <= 0.0001 {
                continue;
            }
            let vv = (fheight - 2.0 * BORDER_WIDTH) / (max_v - min_v);
            let mut priceline: Vec<Vec2> = vec![];
            let mut averageline: Vec<Vec2> = vec![];
            let leftx = if i % 2 == 0 { one_left } else { two_left };
            let liney = fheight * ((i / 2) as f32);

            let last_idx = ticks.len() - 1;

            // 后台打印警示信息。
            let bos = ticks[last_idx].buy + ticks[last_idx].sell;
            let tb = ticks[last_idx].max - ticks[last_idx].min;
            // 一分钟的价格变动范围超过警戒值
            if tb > self.cfg.av_max {
                self.tx
                    .send((
                        fname.to_owned(),
                        1,
                        format!(
                            "当前价格突变， 变化幅度达到 {:.2}, (阈值：{:.2}), B+S:{}",
                            tb, self.cfg.av_max, bos
                        ),
                    ))
                    .unwrap_or_default();
                ticks[last_idx].sign = -1;
            }

            let warning_val = if self.cfg.warnings.contains_key(fname) {
                self.cfg.warnings[fname]
            } else {
                WARNING_AV_DEFUALT
            };

            let warning_bos = if self.cfg.bs_max.contains_key(fname) {
                self.cfg.bs_max[fname]
            } else {
                BUY_AND_SELL_DEFAULT
            };
            // 这里从申请交易量TOP10中取出TOP X，作为警戒值。防止系统设置值太小而造成过多的报警。
            let top_w = if self.bs_tops.contains_key(fname) {
                let fv = &self.bs_tops[fname];
                if fv.len() >= BS_TOP_SIZE {
                    fv[BS_TOP_AS_WARNING].sign
                } else {
                    0
                }
            } else {
                0
            };

            let warning_bos = if warning_bos > top_w {
                warning_bos
            } else {
                top_w
            };

            if bos > warning_bos {
                // 价格偏离均值差 超过警戒值
                let cv = (ticks[last_idx].last_price - ticks[last_idx].average_price).abs();
                if cv > warning_val {
                    self.tx
                        .send((
                            fname.to_owned(),
                            10,
                            format!(
                                "当前价格变化触发阈值({:.2}), {:.2}, 申请交易量触发阈值，B+S:{}",
                                warning_val, ticks[last_idx].last_price, bos
                            ),
                        ))
                        .unwrap_or_default();
                } else {
                    self.tx
                        .send((
                            fname.to_owned(),
                            2,
                            format!(
                                "申请交易超量， B+S:{}，当前价格：{:.2}, 价格幅度未超阈值！",
                                bos, ticks[last_idx].last_price
                            ),
                        ))
                        .unwrap_or_default();
                }
            }

            if mesh_build {
                // 前台显示信息构造
                let text = fname.to_string();
                let info = format!(
                    "Ac: {:.2}, B+S: {}, P: {:.2}",
                    ticks[last_idx].max - ticks[last_idx].min,
                    bos,
                    ticks[last_idx].last_price
                );

                let (aor_text, aor_is_full) = aor.aor_to_result();
                let left_text_x = if ticks.len() < 100 {
                    leftx + BORDER_WIDTH + 100.0 * self.cfg.bar_size
                } else {
                    leftx + BORDER_WIDTH
                };

                self.tests.push((
                    text,
                    left_text_x,
                    liney + 2.0,
                    Color::new(0.8, 0.8, 0.8, 0.2),
                ));
                self.tests.push((
                    info,
                    left_text_x,
                    liney + 80.0,
                    Color::new(0.6, 0.0, 0.0, 0.2),
                ));
                self.tests.push((
                    aor_text,
                    left_text_x,
                    liney + fheight / 2.0,
                    Color::new(0.3, 0.0, 0.0, 0.1),
                ));

                // 绘制价格线和均线
                for idx in 0..ticks.len() {
                    let x = leftx + (idx as f32) * self.cfg.bar_size;
                    let y = liney + fheight - BORDER_WIDTH - (ticks[idx].last_price - min_v) * vv;
                    priceline.push(vec2(x, y));
                    let ay =
                        liney + fheight - BORDER_WIDTH - (ticks[idx].average_price - min_v) * vv;
                    averageline.push(vec2(x, ay));

                    if ticks[idx].sign != 0 {
                        let color = if ticks[idx].sign > 0 {
                            Color::new(0.0, 0.7, 0.0, 0.3)
                        } else {
                            Color::new(0.7, 0.0, 0.0, 0.3)
                        };
                        mesh.circle(
                            DrawMode::Fill(FillOptions::default()),
                            Vec2::new(x, y),
                            SIGN_RADIUS,
                            1.0,
                            color,
                        )?;
                    }
                    // 如果振幅（价格偏离均值的差）超过阈值，并且，交易申请超过阈值，则画"蓝圈"标记, 表面“高价值”交易点
                    let ibos = ticks[idx].buy + ticks[idx].sell;
                    let cv = (ticks[idx].last_price - ticks[idx].average_price).abs();
                    if ibos > warning_bos && cv > warning_val {
                        mesh.circle(
                            DrawMode::Fill(FillOptions::default()),
                            Vec2::new(x, y),
                            SIGN_RADIUS,
                            1.0,
                            Color::new(0.0, 0.0, 1.0, 0.2),
                        )?;
                    }
                }

                if priceline.len() > 1 {
                    mesh.line(&priceline, 1.0, Color::new(0.6, 0.6, 0.6, 1.0))?;
                    mesh.line(&averageline, 1.0, Color::new(1.0, 1.0, 0.0, 1.0))?;
                }
                // 如果是单边，则画X 警示、禁止交易
                if !aor_is_full {
                    let line1: Vec<Vec2> =
                        vec![vec2(leftx, liney), vec2(leftx + one_width, liney + fheight)];
                    let line2: Vec<Vec2> =
                        vec![vec2(leftx, liney + fheight), vec2(leftx + one_width, liney)];
                    mesh.line(&line1, 2.0, Color::new(0.0, 0.36, 0.0, 0.25))?;
                    mesh.line(&line2, 2.0, Color::new(0.0, 0.36, 0.0, 0.25))?;
                }
                // 如果是振幅超过阈值，则画红/绿心警示
                if aor.up_max > warning_val * 1.2 || aor.down_max.abs() > warning_val * 1.2 {
                    let x = leftx + one_width / 2.0;
                    let y = liney + fheight / 2.0;

                    let color = if aor.up_max > warning_val {
                        Color::new(0.25, 0.0, 0.0, 0.25)
                    } else {
                        Color::new(0.0, 0.25, 0.0, 0.25)
                    };
                    mesh.circle(
                        DrawMode::Fill(FillOptions::default()),
                        Vec2::new(x, y),
                        WARNING_RADIUS,
                        1.0,
                        color,
                    )?;
                }
                // TOPS Buy+Sell警示线, 黄色柱
                if self.bs_tops.contains_key(fname) {
                    let tops = self.bs_tops.get(fname).unwrap();
                    let one_h = fheight / 5.0;
                    if tops.len() == BS_TOP_SIZE {
                        for i in 0..5 {
                            let x = leftx + (tops[i].sign as f32) * self.cfg.bar_size;
                            let y0 = liney + fheight - BORDER_WIDTH;
                            let y1 =  liney + fheight  - ((5 -i) as f32  *one_h) ;                                
                            mesh.line(
                                &[vec2(x, y0), vec2(x, y1)],
                                1.0,
                                Color::new(1.0, 0.0, 0.0, 0.25),
                            )?;
                        }
                    }
                }

                // 最高Buy 请求警示线, 蓝色线
                let x = leftx + (max_buy.0 as f32) * self.cfg.bar_size + 2.0;  // 故意偏移2.0
                let y0 = liney + fheight - BORDER_WIDTH ;
                let y1 = liney +  BORDER_WIDTH ;
                mesh.line(
                    &[vec2(x, y0), vec2(x, y1)],
                    1.0,
                    Color::new(0.0, 0.0, 1.0, 0.25),
                )?;
                // 最高 Sell 请求警示线
                let x = leftx + (max_sell.0 as f32) * self.cfg.bar_size - 2.0;  // 故意偏移2.0
                let y0 = liney + fheight - BORDER_WIDTH ;
                let y1 = liney +  BORDER_WIDTH ;
                mesh.line(
                    &[vec2(x, y0), vec2(x, y1)],
                    1.0,
                    Color::new(0.0, 1.0, 0.0, 0.25),
                )?;

                // 最新价格下的Buy+Sell警示线, 画红柱，表明热度
                if bos > warning_bos {
                    let x = leftx + (last_idx as f32) * self.cfg.bar_size;
                    let y =
                        liney + fheight - BORDER_WIDTH - (ticks[last_idx].last_price - min_v) * vv;
                    let y1 = if y - BUY_AND_SELL_WARNING < liney + BORDER_WIDTH {
                        liney + BORDER_WIDTH
                    } else {
                        y - BUY_AND_SELL_WARNING
                    };
                    let y2 = if y + BUY_AND_SELL_WARNING > liney + fheight - BORDER_WIDTH {
                        liney + fheight - BORDER_WIDTH
                    } else {
                        y + BUY_AND_SELL_WARNING
                    };

                    mesh.line(
                        &[vec2(x, y1), vec2(x, y2)],
                        5.0,
                        Color::new(1.0, 0.0, 0.0, 0.20),
                    )?;
                }
             
            }
        }
        if mesh_build {
            // 中间的竖隔离线
            mesh.line(
                &[
                    vec2(self.width / 2.0, BORDER_WIDTH),
                    vec2(self.width / 2.0, self.height - BORDER_WIDTH),
                ],
                1.0,
                Color::new(0.3, 0.3, 0.3, 0.2),
            )?;
            // 行间的横隔离线
            let count = (self.futures.len() - 1) / 2;
            if count >= 1 {
                for i in 0..count {
                    let leftx = one_left;
                    let liney = fheight * (i as f32 + 1.0);
                    let rightx = self.width - BORDER_WIDTH;
                    mesh.line(
                        &[vec2(leftx, liney), vec2(rightx, liney)],
                        1.0,
                        Color::new(0.3, 0.3, 0.3, 0.20),
                    )?;
                }
            }

            let meshes = graphics::Mesh::from_data(ctx, mesh.build());
            self.meshes.push(meshes);
        }
        Ok(())
    }

    fn add_bs_tops(&mut self, fname: &str, idx: usize, mut ti: SimpleTick) -> Result<()> {
        ti.sign = idx as i32;
        ti.buy = ti.buy + ti.sell;
        if self.bs_tops.contains_key(fname) {
            let fv = self.bs_tops.get_mut(fname).unwrap();
            if fv.len() >= BS_TOP_SIZE {
                let last = fv.len() - 1;
                if fv[last].buy < ti.buy {
                    fv.push(ti);
                    fv.sort_by(|a, b| b.buy.cmp(&a.buy));
                }
            } else {
                fv.push(ti);
                fv.sort_by(|a, b| b.buy.cmp(&a.buy));
            }
            while fv.len() > BS_TOP_SIZE {
                fv.remove(fv.len() - 1);
            }
        } else {
            self.bs_tops.insert(fname.to_string(), vec![ti]);
        }
        Ok(())
    }

    fn add_tick(&mut self, fname: &str, ti: SimpleTick) -> Result<()> {
        if self.futures.contains_key(fname) {
            let fv = self.futures.get_mut(fname).unwrap();
            let last = fv.len() - 1;
            if fv[last].update_time == ti.update_time {
                fv[last].last_price = (fv[last].last_price + ti.last_price) / 2.0;
                if fv[last].max < ti.max {
                    fv[last].max = ti.max;
                }
                if fv[last].min > ti.min {
                    fv[last].min = ti.min;
                }
                fv[last].buy += ti.buy;
                fv[last].sell += ti.sell;
            } else if ti.update_time > fv[last].update_time {
                if fv.len() > 60 * 4 {
                    self.tx
                        .send((
                            fname.to_owned(),
                            0,
                            format!("too many ticks, Please press DELETE to clear!"),
                        ))
                        .unwrap_or_default();
                } else {
                    fv.push(ti);
                    // 此时，表示新增的tick是第一个tick，读取上一个完整的tick, 放入bs_tops
                    if fv.len() > 1 {
                        let id = fv.len() - 2;
                        let lt = fv[id].clone();
                        self.add_bs_tops(fname, id, lt)?;
                    }
                }
            }
        } else {
            let v = vec![ti];
            self.futures.insert(fname.to_string(), v);
        }
        Ok(())
    }

    pub fn receive_message(&mut self) -> Result<()> {
        let mut buf = [0; 1024];
        let (amt, _src) = self.socket.recv_from(&mut buf)?;
        //println!("Received {} bytes from {}", amt, src);
        if amt == 0 {
            bail!("received 0 bytes")
        }

        let src = String::from_utf8(buf[..amt].to_vec())?;
        debug!(target:"message", "received: {}",  src);
        let vs: Vec<&str> = src.split(';').collect();
        if vs.len() != FOXY_TICK_LEN {
            bail!("invalid message");
        }
        // let buy_or_sell: i32 = vs[1].parse::<i32>()?;

        let mut tick = SimpleTick {
            average_price: vs[0].parse::<f32>()?,
            buy: vs[1].parse::<i32>()?,
            sell: vs[2].parse::<i32>()?,
            last_price: vs[4].parse::<f32>()?,
            update_time: vs[7][0..5].to_string(),
            volume: vs[8].parse::<i32>()?,
            max: 0.0,
            min: 0.0,
            radius: 0.0,
            sign: 0,
        };
        tick.max = tick.last_price;
        tick.min = tick.last_price;
        // 某些商品，如塑料l2401,averaged_price是一手（5吨）的均价，需要除以参数5.0，计算出每吨的均价。
        let futures_name = vs[3];
        if self.cfg.avargs.contains_key(futures_name) && self.cfg.avargs[futures_name] != 0.0 {
            tick.average_price = tick.average_price / self.cfg.avargs[futures_name];
        }
        self.add_tick(futures_name, tick)?;
        Ok(())
    }

    /**
     * 清除数据,并重置系统初始值
     */
    fn clean_data(&mut self) {
        self.meshes.clear();
        self.futures.clear();
        self.tests.clear();
        self.bs_tops.clear();
        self.futures_index.clear();
    }
    /*
    针对指定fidx的商品的 idx 位置的tick 进行标记，如果没有，则返回false。
    */
    fn set_tick_sign(&mut self, fidx: usize, idx: usize, sign: i32) -> Result<bool> {
        if fidx >= self.futures_index.len() {
            bail!("index out of range, or not found");
        }
        let mut ret = false;
        debug!(target:"cunder", "futures_index: {:#?}", &self.futures_index);

        let fv = self.futures.get_mut(&self.futures_index[fidx]).unwrap();
        if fv.len() > idx {
            if fv[idx].sign != sign {
                fv[idx].sign = sign;
                ret = true;
            }
        }
        Ok(ret)
    }
    /*
    对指定fidx商品的 idx 位置的tick 进行标记清楚，犹豫鼠标点击做出的判断存在误差，所以对idx进行了扩大检测，左右+2进行检测
    ，如果sign>0，则重置sign=0，并返回true,
    ，如果没有，则返回false。
    */
    fn clean_tick_sign(&mut self, fidx: usize, idx: usize) -> Result<bool> {
        if fidx >= self.futures_index.len() {
            bail!("index out of range, or not found");
        }
        let fv = self.futures.get_mut(&self.futures_index[fidx]).unwrap();
        if idx >= fv.len() {
            bail!("index out of range, or not found");
        }

        let more = 3;
        let max_idx = fv.len() - 1;
        let max_idx = if (max_idx - idx) >= more {
            idx + more
        } else {
            max_idx
        };
        let min_idx = if (idx) >= more { idx - more } else { 0 };
        let mut reset = false;
        for i in min_idx..max_idx {
            if fv[i].sign != 0 {
                reset = true;
                fv[i].sign = 0;
            }
        }
        return Ok(reset);
    }

    /*
    保存futures数据到文件中，
    格式为：update_time,last_price, max, min, updown, average_price, volume, radius,
     */
    fn write_cvs(&self) -> Result<()> {
        let globalname = PathBuf::from(&self.work_path).join(FUTURES_INDEX_NAME);
        if Path::new(&globalname).exists() {
            std::fs::remove_file(&globalname)?;
        }

        let mut wtr = Writer::from_path(&globalname)?;
        // 第一行通常为字段名称，读取时默认忽略第一行
        wtr.write_record(&["futures_code_name"])?;
        for (k, _) in self.futures.iter() {
            wtr.write_record(&[k.clone()])?;
        }
        wtr.flush()?;

        let write_func = |filename: &str, v: &Vec<SimpleTick>| -> Result<()> {
            if Path::new(&filename).exists() {
                std::fs::remove_file(&filename)?;
            }
            let mut wtr = Writer::from_path(&filename)?;
            wtr.write_record(&[
                "update_time",
                "last_price",
                "max",
                "min",
                "buy",
                "sell",
                "average_price",
                "volume",
                "radius",
            ])?;
            for x in v.iter() {
                wtr.write_record(&[
                    x.update_time.clone(),
                    x.last_price.to_string(),
                    x.max.to_string(),
                    x.min.to_string(),
                    x.buy.to_string(),
                    x.sell.to_string(),
                    x.average_price.to_string(),
                    x.volume.to_string(),
                    x.radius.to_string(),
                ])?;
            }
            wtr.flush()?;
            Ok(())
        };

        for (k, v) in self.futures.iter() {
            let wpath = PathBuf::from(&self.work_path);
            write_func(
                wpath
                    .join(format!("{}.csv", &k))
                    .as_os_str()
                    .to_str()
                    .unwrap_or_default(),
                v,
            )?;
        }
        Ok(())
    }
    /*
    从csv文件中读取数据，并保存到futures中，
    格式为：update_time,last_price, max, min, updown, average_price, volume, radius,
    */
    pub fn read_cvs(&mut self, fname: &str) -> Result<()> {
        self.clean_data();
        let mut futures: Vec<String> = vec![];
        let parent = Path::new(fname).parent().unwrap();
        if !Path::new(fname).exists() {
            bail!("file not found");
        }
        let mut rdr = csv::ReaderBuilder::new().from_path(fname)?;
        for result in rdr.records() {
            let record = result?;
            futures.push(record[0].to_string());
        }
        println!("futures:{}\n {:#?}", futures.len(), futures);

        let read_func = |filename: &str, v: &mut Vec<SimpleTick>| -> Result<()> {
            if Path::new(filename).exists() {
                let mut rdr = csv::ReaderBuilder::new().from_path(filename)?;
                for result in rdr.records() {
                    let record = result?;
                    let mut tick = SimpleTick::default();
                    tick.update_time = record[0].to_string();
                    tick.last_price = record[1].to_string().parse().unwrap_or(0.0);
                    tick.max = record[2].to_string().parse().unwrap_or(0.0);
                    tick.min = record[3].to_string().parse().unwrap_or(0.0);
                    tick.buy = record[4].to_string().parse().unwrap_or(0);
                    tick.sell = record[5].to_string().parse().unwrap_or(0);
                    tick.average_price = record[6].to_string().parse().unwrap_or(0.0);
                    tick.volume = record[7].to_string().parse().unwrap_or(0);
                    tick.radius = record[8].to_string().parse().unwrap_or(0.0);
                    v.push(tick);
                }
            }
            Ok(())
        };

        for f in futures.iter() {
            let wpath = PathBuf::from(&parent);
            let mut ticks: Vec<SimpleTick> = vec![];
            read_func(
                wpath
                    .join(format!("{}.csv", &f))
                    .as_os_str()
                    .to_str()
                    .unwrap_or_default(),
                &mut ticks,
            )?;
            // 构造 TOPS
            for i in 0..ticks.len() {
                let tt = ticks[i].clone();
                self.add_bs_tops(f, i, tt)?;
            }
            self.futures.insert(f.clone(), ticks);
        }

        Ok(())
    }
    fn print_bs_top(&self) {
        if self.bs_tops.len() == 0 {
            println!("bs_tops is empty");
            return;
        }
        for (k, v) in self.bs_tops.iter() {
            println!("\n{}, sign = buy + sell", k);
            for x in 0..v.len() {
                println!(
                    " NO.{}  time = {}, lastprice = {:.2}, average = {:.2}, B+S = {}",
                    x + 1,
                    &v[x].update_time,
                    v[x].last_price,
                    v[x].average_price,
                    v[x].buy
                );
            }
        }
    }
    fn update_config(&mut self) {
        let cfg = Config::load(&self.cfg_file).unwrap_or_default();
        self.cfg = cfg;
    }

}

impl event::EventHandler<ggez::GameError> for Cunder {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // 每秒执行一次
        const DESIRED_FPS: u32 = 1;
        if let Err(er) = self.receive_message() {
            debug!(target:"receive", "receive message error {:#?}", er);
        } else {
            if ctx.time.check_update_time(DESIRED_FPS) {
                self.build_meshes(ctx, false).unwrap_or_default();
            }
        }

        if self.meshes.len() == 0 && self.futures.len() > 0 {
            self.build_meshes(ctx, true).unwrap_or_default();
        }

        ggez::error::GameResult::Ok(())
    }

    //
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.0, 0.0, 0.0, 1.0]));
        canvas.set_default_sampler();

        if self.meshes.len() > 0 {
            for mesh in &self.meshes {
                canvas.draw(mesh, graphics::DrawParam::new());
            }

            for (text, x, y, color) in &self.tests {
                canvas.draw(
                    graphics::Text::new(text).set_scale(self.cfg.font_size),
                    graphics::DrawParam::from(Vec2::new(*x, *y)).color(*color),
                );
            }
        } else {
            canvas.draw(
                graphics::Text::new("NO SIGNAL FROM CTP, WAIT FOR A WHILE...!").set_scale(60.),
                graphics::DrawParam::from(Vec2::new(200.0, 300.0)).color(Color::RED),
            );
        }

        // Finished drawing, show it all on the screen!
        canvas.finish(ctx)?;
        GameResult::Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(keyboard::KeyCode::Escape) => {
                println!("Escape pressed")
            },
            Some(keyboard::KeyCode::Delete) => {
                self.clean_data();
                println!("{}", Red.paint("CLEAN ALL DATA, please wait for a while!"));
            },
            Some(keyboard::KeyCode::F1) => {
                println!("{}\n{:#?}", Red.paint("config as follows"), self.cfg);
            },
            Some(keyboard::KeyCode::F2) => {
                if self.futures.len() < 1 {
                    error!("no futures, mybe Market Closed?");
                    return GameResult::Ok(());
                }
                for (k, v) in &self.futures {
                    println!("\n{}", k);
                    let idx = if v.len() > 10 { v.len() - 10 } else { 0 };
                    for t in idx..v.len() {
                        println!(" {}. {:?}", t + 1, v[t]);
                    }
                }
            },
            Some(keyboard::KeyCode::F3) => {
                self.print_bs_top();
            },
            Some(keyboard::KeyCode::F4) => {
                self.update_config();
                println!("new config as follows\n{:#?}", self.cfg);
            },
            Some(keyboard::KeyCode::F5) => {
                self.write_cvs().unwrap_or_default();
            },
            Some(keyboard::KeyCode::Q) => {
                println!("{}", Red.paint("Save data & QUIT"));
                self.write_cvs().unwrap_or_default();
                ctx.request_quit()
            },
            _ => {}
        }
        GameResult::Ok(())
    }

    /*
    增加，删除下单标记点；左键双击，增加标记点；右键双击，删除当前位置的标记点；

    */
    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        let fsize = self.futures.len();
        if fsize < 1 {
            error!("NO DATA, mybe Market Closed?");
            return GameResult::Ok(());
        }

        let row_n = (fsize - 1) / 2 + 1;
        let yh = self.height / (row_n as f32);
        let row = (y / yh) as usize;

        let (idx, idf) = if x < self.width / 2.0 {
            (((x - BORDER_WIDTH) / self.cfg.bar_size) as usize, 2 * row)
        } else {
            (
                ((x - self.width / 2.0 - BORDER_WIDTH) / self.cfg.bar_size) as usize,
                2 * row + 1,
            )
        };

        let now = Utc::now();
        let mut doubleclick = false;
        let mut rebuild_mesh = false;
        match button {
            MouseButton::Left => {
                if now
                    .signed_duration_since(self.last_click.time)
                    .num_seconds()
                    < 2
                    && self.last_click.button == MouseButton::Left
                    && (self.last_click.x - x).abs() < 5.0
                    && (self.last_click.y - y).abs() < 5.0
                {
                    doubleclick = true;
                    error!("增加标记点，futures_idx = {}, ticks_idx = {}", idf, idx);
                    if idf < self.futures.len() {
                        match self.set_tick_sign(idf, idx, 1) {
                            Result::Ok(setok) => rebuild_mesh = setok,
                            Err(e) => {
                                error!("{}", e);
                            }
                        }
                    }
                }
            }
            MouseButton::Right => {
                if now
                    .signed_duration_since(self.last_click.time)
                    .num_seconds()
                    < 2
                    && (self.last_click.x - x).abs() < 5.0
                    && (self.last_click.y - y).abs() < 5.0
                {
                    doubleclick = true;
                    if self.last_click.button == MouseButton::Left {
                        println!("left + right click");
                    } else if self.last_click.button == MouseButton::Right {
                        info!(
                            "删除标记点，futures_idx = {}, ticks_idx = [{},{}]",
                            idf,
                            idx - 3,
                            idx + 3
                        );
                        if idf < self.futures.len() {
                            match self.clean_tick_sign(idf, idx) {
                                Result::Ok(setok) => rebuild_mesh = setok,
                                Err(e) => {
                                    error!("{}", e);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        self.last_click = if doubleclick {
            Click {
                x,
                y,
                time: now,
                button: MouseButton::Other(0),
            }
        } else {
            Click {
                x,
                y,
                time: now,
                button,
            }
        };
        if rebuild_mesh {
            self.build_meshes(ctx, true).unwrap_or_default();
        }
        GameResult::Ok(())
    }
    fn quit_event(
        &mut self,
        _ctx: &mut Context,
    ) -> std::prelude::v1::Result<bool, ggez::GameError> {
        self.tx
            .send(("exit".to_owned(), 0, "bye".to_owned()))
            .unwrap_or_default();
        GameResult::Ok(false)
    }
}
