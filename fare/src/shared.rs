/*!
shared handle for multi-thread
- 维护配置文件和行情数据
- 使用std::sync::Rwlock, 而没有使用tokio::sync::RwLock, 因为虽然前者会导致阻塞，但是因为egui是同步的，所以更方便调用。对于程序应用场景而言，阻塞没有影响。

*/

use crate::*;
use csv::Writer;
/**
多线程共享数据句柄
 */
pub struct Shared {
    // for websocket in chatgpt viewer
    pub udp_addr: String,
    pub config: Config,
    // file_name: String,
    // file_path: String,
    pub futures: Futures,
}

/**
期货商品信息的结构体
*/
pub struct Futures {
    pub data: std::sync::RwLock<Vec<SimpleTick>>,
    pub topx: std::sync::RwLock<Topx>,
    pub futures_name: std::sync::RwLock<String>,
}

// 默认的TOP数量
const MAX_TOPX: usize = 10;

#[derive(Debug, Clone, Default)]
/**
7大元素的TOP集合（成交量，买申请，卖申请，总申请，成交率，价格最高，价格最低）
数值内容是二元组，第一元素是对应的数值，第二元素对应的是index，即当前值在整个Tick集合中的索引位置
*/
pub struct Topx {
    /** 外部tick数据集的下一个更新起始位（表示此前已经处理过，无需处理），特别注意，是 next index  */
    pub idx: usize,
    /** 成交量 */
    pub volume: Vec<(i32, usize)>,
    /** 买申请数量 */
    pub buy: Vec<(i32, usize)>,
    /** 卖申请数量 */
    pub sell: Vec<(i32, usize)>,
    /** 买卖申请数量和 */
    pub bs: Vec<(i32, usize)>,
    /** 成交量除以买卖申请数量和，可以理解为成交率 */
    pub vbs: Vec<(f32, usize)>,
    /** 价格最大值 */
    pub maxv: Vec<(f32, usize)>,
    /** 价格最小值 */
    pub minv: Vec<(f32, usize)>,
}

impl Topx {
    pub fn new() -> Self {
        Self {
            idx: 0,
            volume: vec![],
            buy: vec![],
            sell: vec![],
            bs: vec![],
            vbs: vec![],
            maxv: vec![],
            minv: vec![],
        }
    }
    
    /**
    clone TOP集合，参数为指定的TOP集合的大小。如果大小超过MAX_TOPX，则返回MAX_TOPX个元素。
    - 因为数据集合小，clone的性能损耗低。
    - clone而非引用，减少与其他元素的锁竞争或间接锁竞争。
    */
    pub fn get_tops(
        &self,
        size: usize,
    ) -> (
        Vec<(f32, usize)>,
        Vec<(f32, usize)>,
        Vec<(i32, usize)>,
        Vec<(i32, usize)>,
        Vec<(i32, usize)>,
        Vec<(i32, usize)>,
        Vec<(f32, usize)>,
    ) {
        let maxv = if size < self.maxv.len() {
            self.maxv[0..size].to_vec()
        } else {
            self.maxv.clone()
        };

        let minv = if size < self.minv.len() {
            self.minv[0..size].to_vec()
        } else {
            self.minv.clone()
        };

        let volume = if size < self.volume.len() {
            self.volume[0..size].to_vec()
        } else {
            self.volume.clone()
        };

        let buy = if size < self.buy.len() {
            self.buy[0..size].to_vec()
        } else {
            self.buy.clone()
        };

        let sell = if size < self.sell.len() {
            self.sell[0..size].to_vec()
        } else {
            self.sell.clone()
        };

        let bs = if size < self.bs.len() {
            self.bs[0..size].to_vec()
        } else {
            self.bs.clone()
        };

        let vbs = if size < self.vbs.len() {
            self.vbs[0..size].to_vec()
        } else {
            self.vbs.clone()
        };

        (maxv, minv, volume, buy, sell, bs, vbs)
    }

    /**
    针对TOP集，增加新的tick, 以便于更新所有的TOP集数据。
    参数tick为当前tick,
    volume为计算好的volume,
    idx为当前tick所在的index位置。
    */
    pub fn next_ticks(&mut self, tick: &SimpleTick, volume: i32, idx: usize) {
        if self.volume.len() == 0 {
            self.volume.push((volume, idx));
        } else {
            if volume > self.volume[self.volume.len() - 1].0 || self.volume.len() < MAX_TOPX {
                self.volume.push((volume, idx))
            }
        }

        if self.buy.len() == 0 {
            self.buy.push((tick.buy, idx));
        } else {
            if tick.buy > self.buy[self.buy.len() - 1].0 || self.buy.len() < MAX_TOPX {
                self.buy.push((tick.buy, idx))
            }
        }

        if self.sell.len() == 0 {
            self.sell.push((tick.sell, idx));
        } else {
            if tick.sell > self.sell[self.sell.len() - 1].0 || self.sell.len() < MAX_TOPX {
                self.sell.push((tick.sell, idx))
            }
        }

        if self.bs.len() == 0 {
            self.bs.push((tick.buy + tick.sell, idx));
        } else {
            if (tick.buy + tick.sell) > self.bs[self.bs.len() - 1].0 || self.bs.len() < MAX_TOPX {
                self.bs.push((tick.buy + tick.sell, idx))
            }
        }

        let vv = if tick.buy + tick.sell > 0 {
            volume as f32 / (tick.buy + tick.sell) as f32
        } else {
            0.0
        };
        if self.vbs.len() == 0 {
            self.vbs.push((vv, idx));
        } else {
            if vv > self.vbs[self.vbs.len() - 1].0 || self.vbs.len() < MAX_TOPX {
                self.vbs.push((vv, idx))
            }
        }

        if self.maxv.len() == 0 {
            self.maxv.push((tick.last_price, idx));
        } else {
            if tick.last_price > self.maxv[self.maxv.len() - 1].0 || self.maxv.len() < MAX_TOPX {
                self.maxv.push((tick.last_price, idx))
            }
        }

        if self.minv.len() == 0 {
            self.minv.push((tick.last_price, idx));
        } else {
            if tick.last_price < self.minv[self.minv.len() - 1].0 || self.minv.len() < MAX_TOPX {
                self.minv.push((tick.last_price, idx))
            }
        }

        //
        self.volume.sort_by(|a, b| b.0.cmp(&a.0));
        if self.volume.len() > MAX_TOPX {
            self.volume.remove(self.volume.len() - 1);
        }

        self.buy.sort_by(|a, b| b.0.cmp(&a.0));
        if self.buy.len() > MAX_TOPX {
            self.buy.remove(self.buy.len() - 1);
        }

        self.sell.sort_by(|a, b| b.0.cmp(&a.0));
        if self.sell.len() > MAX_TOPX {
            self.sell.remove(self.sell.len() - 1);
        }

        self.bs.sort_by(|a, b| b.0.cmp(&a.0));
        if self.bs.len() > MAX_TOPX {
            self.bs.remove(self.bs.len() - 1);
        }

        self.vbs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if self.vbs.len() > MAX_TOPX {
            self.vbs.remove(self.vbs.len() - 1);
        }

        self.maxv.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        if self.maxv.len() > MAX_TOPX {
            self.maxv.remove(self.maxv.len() - 1);
        }

        self.minv.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        if self.minv.len() > MAX_TOPX {
            self.minv.remove(self.minv.len() - 1);
        }
        // 更新索引，指向下一个tick数据的index(初始化时为0)
        self.idx = idx + 1;
    }

    pub fn clear(&mut self) {
        self.idx = 0;
        self.volume.clear();
        self.buy.clear();
        self.sell.clear();
        self.bs.clear();
        self.vbs.clear();
        self.maxv.clear();
        self.minv.clear();
    }
}

impl Shared {
    pub fn new(cfg: &str, addr: &str) -> Self {
        let mut config = Config::load(cfg).unwrap_or_default();
        if config.top_count == 0 {
            config.top_count = 1;
        }
        if config.warn_value < 0.1 {
            config.warn_value = 10.0;
        }
        if config.average_package < 1 {
            config.average_package = 1;
        }
        Self {
            udp_addr: addr.to_owned(),
            config,
           // file_path: savepath.to_owned(),
            futures: Futures {
                futures_name: std::sync::RwLock::new("".to_owned()),
                data: std::sync::RwLock::new(vec![]),
                topx: std::sync::RwLock::new(Topx::new()),
            },
        }
    }

    pub fn set_futures_name(&self, name: &str) {
        if name.len() == 0 {
            return;
        }
        let mut w = self.futures.futures_name.write().unwrap();
        (*w).clear();
        (*w).push_str(name);
    }

    /**
    获取当前的商品的名称
    */
    pub fn get_futures_name(&self) -> String {
        let r = self.futures.futures_name.read().unwrap();
        return (*r).clone();
    }

    /**
    获取Tick数据的长度
    */
    pub fn get_size(&self) -> usize {
        let r = self.futures.data.read().unwrap();
        (*r).len()
    }

    /**
    从csv文件中读取数据，并保存到futures中，
    格式为：update_time,last_price, average_price, volume,
    */
    pub fn read_cvs(&mut self, fname: &str) -> Result<()> {
        if !Path::new(fname).exists() {
            bail!("file not found");
        }
        let mut rdr = csv::ReaderBuilder::new().from_path(fname)?;
        self.clear();

        let mut w = self.futures.data.write().unwrap();
        for result in rdr.records() {
            let record = result?;
            let mut tick = SimpleTick::default();
            tick.update_time = record[0].to_string();
            tick.last_price = record[1].to_string().parse().unwrap_or(0.0);
            tick.average_price = record[2].to_string().parse().unwrap_or(0.0);
            tick.buy = record[3].to_string().parse().unwrap_or(0);
            tick.sell = record[4].to_string().parse().unwrap_or(0);
            tick.volume = record[5].to_string().parse().unwrap_or(0);
            (*w).push(tick);
        }
        drop(w);
        // 构造Topx，否则plotters页面无法获取到max_v, min_v, volume, buy, sell, bs, vbs。
        self.next();
        let fname =PathBuf::from(fname).file_name().unwrap().to_str().unwrap().to_owned();
        let f_v: Vec<&str> = fname.split("-").collect();
        if f_v.len() > 0 {
            self.set_futures_name(f_v[0]);
        }

        Ok(())
    }

    /**
    保存futures数据到文件中，
    格式为：update_time,last_price, average_price, buy, sell, volume,
     */
    pub fn write_cvs(&self, futures_name: &str) -> Result<()> {
    
        let datapath  = Local::now().format("%Y%m%d").to_string();
        if !Path::new(&datapath).exists() {
            std::fs::create_dir_all(&datapath)?;
        }

        let r = self.futures.data.read().unwrap();
        if (*r).len() == 0 {
            return Ok(());
        }
        let current_time = Local::now(); // Utc::now();
        let fname = if futures_name.len() == 0 {
            "unkown"
        } else {
            futures_name
        };
        let fname = format!("{}-{}.csv", fname, current_time.format("%H%M%S"));
        let filename = PathBuf::from(&datapath).join(fname);
        if Path::new(&filename).exists() {
            std::fs::remove_file(&filename)?;
        }
        let mut wtr = Writer::from_path(&filename)?;
        wtr.write_record(&[
            "update_time",
            "last_price",
            "average_price",
            "buy",
            "sell",
            "volume",
        ])?;
        // let r = self.futures.data.read().unwrap();

        for x in (*r).iter() {
            wtr.write_record(&[
                x.update_time.clone(),
                x.last_price.to_string(),
                x.average_price.to_string(),
                x.buy.to_string(),
                x.sell.to_string(),
                x.volume.to_string(),
            ])?;
        }
        wtr.flush()?;
        Ok(())
    }

    /**
    打印配置信息
     */
    pub fn print_config(&self) {
        println!("\nconfig:");
        println!("{:?}", self.config);      
        println!("futures code = {:?}", self.get_futures_name());
        println!("listen udp addr = {}", &self.udp_addr);
    }
    /**
    直接在后台打印最新的n条tick数据。
     */
    pub fn print_last_data(&self, size: usize) {
        let rds = self.futures.data.read().unwrap();
        let len = (*rds).len();
        if len == 0 {
            println!("No data");
            return;
        }
        println!("\nLast {} data as follows:", size);
        let c = if len > size { len - size } else { 0 };
        for i in c..len {
            let vol = if i >= 1 {
                (*rds)[i].volume - (*rds)[i - 1].volume
            } else {
                0
            };
            println!(
                "{}, {:.2}, av = {:.2}, B = {}, S = {}, volume = {}",
                &(*rds)[i].update_time,
                &(*rds)[i].last_price,
                &(*rds)[i].average_price,
                &(*rds)[i].buy,
                &(*rds)[i].sell,
                vol
            );
        }
    }

    pub fn get_topx(
        &self,
        top_size: usize,
    ) -> (
        Vec<(f32, usize)>,
        Vec<(f32, usize)>,
        Vec<(i32, usize)>,
        Vec<(i32, usize)>,
        Vec<(i32, usize)>,
        Vec<(i32, usize)>,
        Vec<(f32, usize)>,
    ) {
        let topx = self.futures.topx.read().unwrap();
        return topx.get_tops(top_size);
    }

    fn next(&self) {
        let futures = &(*(self.futures.data.read().unwrap()));
        let len = futures.len();
        let mut topx = self.futures.topx.write().unwrap();
        let done_idx = topx.idx;
        if len == 0 || done_idx >= len - 1 {
            // 注意，len-1, 将不会处理最后一个数据，因为此数据还在更新，不是最终数据！！
            return;
        }

        for i in done_idx..len - 1 {
            // 注意，len-1, 将不会处理最后一个数据，因为此数据还在更新，不是最终数据！！
            let vol = if i == 0 {
                0
            } else {
                futures[i].volume - futures[i - 1].volume
            };
            (*topx).next_ticks(&futures[i], vol, i);
        }
    }

   pub fn strategy(&self) {
        let r= self.futures.data.read().unwrap();
        let last = (*r).len() - 1;           
        drop(r);
        if last < 100 {  // 数据量太少，误判不准确，不处理
            return;
        }        

        let min_step:usize =  if last < 300 {  1  } else {  2 };
        let topx = self.futures.topx.read().unwrap();     
        /*
        这里曾有个bug，很有意思：
        if x.1 - last  <= 1 {
            return true;
        }

        结果，系统发送crash！
        原因就是两个usize数据的相减，出现了小于0的结果，而系统默认结果也应该是usize，所以触发了panic。
        解决方法是，倒置，因为last肯定是最大的！！！
        */   
        let buy_in = topx.buy.iter().any(|x| {
            if last - x.1  <= min_step {
                return true;
            }
            false
        });

        let sell_in = topx.sell.iter().any(|x| {
            if last - x.1  <= min_step {
                return true;
            }
            false
        });

        let volume_in = topx.volume.iter().any(|x| {
            if last - x.1  <= min_step {
                return true;
            }
            false
        });        

        if (sell_in && volume_in) || (buy_in && volume_in) {
            error!("cross over sell, buy, and volume, NOW is a key point! {}", Local::now().format("%H:%M:%S") )            
        }   
    }
    /**
    直接在后台打印top n条数据, 包括TOP volume，TOP buy, TOP sell, TOP V/a
    */
    pub fn print_tops(&self, top_size: usize) {
        let r = self.futures.data.read().unwrap();
        let len = (*r).len();
        if len == 0 {
            println!("No data");
            return;
        }
        // let (maxv, minv, volume, buy, sell, bs, vbs) = self.get_tops(&(*r), top_size);
        let (maxv, minv, volume, buy, sell, bs, vbs) = self.get_topx(top_size);
        println!("\ntop volume:");
        volume.iter().enumerate().for_each(|(x, y)| {
            println!("{}. {}: {}", x, (*r)[y.1].update_time, y.0);
        });

        println!("\ntop buy:");
        buy.iter().enumerate().for_each(|(x, y)| {
            println!("{}. {}: {}", x, (*r)[y.1].update_time, y.0);
        });

        println!("\ntop sell:");
        sell.iter().enumerate().for_each(|(x, y)| {
            println!("{}. {}: {}", x, (*r)[y.1].update_time, y.0);
        });

        println!("\ntop max:");
        maxv.iter().enumerate().for_each(|(x, y)| {
            println!("{}. {}: {:.2}", x, (*r)[y.1].update_time, y.0);
        });

        println!("\ntop min:");
        minv.iter().enumerate().for_each(|(x, y)| {
            println!("{}. {}: {:.2}", x, (*r)[y.1].update_time, y.0);
        });

        println!("\ntop buy+sell:");
        bs.iter().enumerate().for_each(|(x, y)| {
            println!("{}. {}: {}", x, (*r)[y.1].update_time, y.0);
        });

        println!("\ntop V/a");
        vbs.iter().enumerate().for_each(|(x, y)| {
            println!("{}. {}: {:.2}", x, (*r)[y.1].update_time, y.0);
        });
    }

    pub fn add_tick(&self, tick: SimpleTick) -> bool {
        let mut done = false;
        let mut w = self.futures.data.write().unwrap();
        if w.len() == 0 {
            // 第一个数据将负责设置futures_name
            (*w).push(tick);
            done = true;
        } else {
            // 同一品种的tick数据
            let last = w.len() - 1;
            if (*w)[last].update_time == tick.update_time {
                (*w)[last].last_price = ((*w)[last].last_price + tick.last_price) / 2.0;
                (*w)[last].buy += tick.buy;
                (*w)[last].sell += tick.sell;
            } else if (*w)[last].update_time < tick.update_time {
                (*w).push(tick);
                done = true;
            }
        }
        // 强行释放锁资源，否则存在死锁
        drop(w);
        if done {
            // 新增一个tick数据，更新topx
            self.next();
        }
        done
    }
    pub fn clear(&self) {
        let mut w = self.futures.data.write().unwrap();
        (*w).clear();
        drop(w);

        let mut w = self.futures.topx.write().unwrap();
        (*w).clear();
        drop(w);

        let mut w = self.futures.futures_name.write().unwrap();
        (*w).clear();
    }

    pub fn report(&self) {
        let r = self.futures.data.read().unwrap();
        if r.len() == 0 {
            return ;
        }

        let mut sell  = 0;
        let mut buy  = 0;
        let mut bs  =  0;
        (*r).iter().for_each(|x| {
            sell += x.sell;
            buy += x.buy;
            bs += x.buy + x.sell;
        });
        let len = (*r).len();
        let volumme = (*r)[len-1].volume - (*r)[0].volume;
        let updown = if (*r)[len-1].average_price - (*r)[0].average_price > 0.0 {
            "up"
        } else {
            "down"
        };      
        let bs_str = if buy > sell {
            format!("buy/sell = {:.2}", buy as f32 / sell as f32)
        } else if buy < sell {
            format!("sell/buy = {:.2}", sell as f32 / buy as f32)
        } else {
            "equal!".to_owned()
        };

        println!(
            "{}, {}, buy: {}, sell: {}, volume: {}, bs: {}, v/a: {:.2}%",
            updown, 
            bs_str,                       
            buy,
            sell,
            volumme,
            bs,
            100.0*volumme as f32 / (buy + sell) as f32);
    }
}
