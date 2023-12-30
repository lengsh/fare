/*! 期货多商品行情跟踪监视助手，cunder -c config.json -e error -f 20231212/futures_index.csv -p 12345

 - 借助ggez进行图形画展示，启动线程通过channel接收信息、并控制频度打印信息（控制单个商品信息霸屏的问题）。
 - 支持多个商品同时监控，通过配置文件设置各自阈值信息。
 - 支持CSV格式数据保存，便于二次分析（如，ppng），和读入再展示。
 - 接收UDP数据依赖foxy,格式：
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
use ggez::{event, GameResult};

//use chrono::prelude::*;
//use std::time::Duration;
use std::{path::{Path, PathBuf}, collections::HashMap};
use time::{format_description, UtcOffset};
use chrono::prelude::*;
//use tracing::{debug, error, info, trace, warn};
use ansi_term::Colour::*;
use clap::Parser;
use tracing_subscriber::{
    self, filter::EnvFilter, fmt, fmt::time::OffsetTime, layer::SubscriberExt,
    util::SubscriberInitExt, Registry,
};
use std::sync::mpsc;
use std::thread;
use tracing::{debug, error, info, trace, warn};

// use anyhow::{anyhow, bail};
mod config;
pub(crate) use config::*;
mod cunder;
pub(crate) use cunder::*;
mod define;
pub(crate) use define::*;
mod aor;
pub(crate) use aor::*;

// type Result<T> = anyhow::Result<T>;

#[derive(Parser, Debug)]
#[clap(
    name = "Cunderr",
    version = "1.0",
    author = "lengss",
    about = "futures assistant"
)]
pub struct Cli {
    /// Sets a custom trace filter, such as info|debug|error|...
    #[clap(name = "trace", short('e'), long, default_value = "error")]
    trace: String,
    #[clap(name = "config", short, long, default_value = "config.json")]
    cfg: String,
    #[clap(name = "datapath", short, long, default_value = "")]
    datapath: String,
    #[clap(name = "futures_index", short, long, default_value = "")]
    futures: String,
    #[clap(short, long, default_value_t = 0)]
    port: u16,
}

pub fn main() -> GameResult {
    let cli = Cli::parse();
    tracelog_init(&cli.trace);
    
    let datapath = 
    if cli.datapath.len() == 0 {
        let current_time = Utc::now();
        current_time.format("%Y%m%d").to_string()
    }else{
        cli.datapath
    };

    if !Path::new(&datapath).exists() {
        std::fs::create_dir_all(&datapath)?;
    }

    let cfg_file = PathBuf::from(&cli.cfg);
    let mut cfg = if cfg_file.exists() == false {
        Config::new_default(&cli.cfg).expect("create default config error")
    } else {
        Config::load(&cli.cfg).expect("load config error")
    };
    println!("read config from {}", Red.paint(&cli.cfg));
    if cli.port > 0 {
        cfg.udp_port = cli.port;
    }
    if cfg.warning_interval < 1 {
        cfg.warning_interval = 60;
    }
    let warning_interval = cfg.warning_interval;

    if cfg.height < WIN_MIN_HEIGHT {
        cfg.height = WIN_MIN_HEIGHT;
    }
    if cfg.bar_size < 1.0 {
        cfg.bar_size = 1.0;
    }

    if cfg.font_size < 10.0 {
        cfg.font_size = 10.0;
    }

    let width = cfg.bar_size * 60.0 * 4.5 * 2.0;
    let height = cfg.height;
    let title = "strategy assistant for futrues".to_owned();

    println!("{:?}", cfg);

    let cb = ggez::ContextBuilder::new("futures", "lengss")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title(&title))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(width, cfg.height));
    // And finally we attempt to build the context and create the window. If it fails, we panic with the message
    // "Failed to build ggez context"
    //.add_resource_path(resource_dir);
    // let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    let (tx, rx) = mpsc::channel();

    let (mut ctx, events_loop) = cb.build()?;
    let mut state = Cunder::new(&mut ctx, cfg,  cli.cfg.clone(), datapath, height, width, tx).unwrap();
    if Path::new(&cli.futures).exists() {
       state.read_cvs(&cli.futures).unwrap_or_default();
    }

    thread::spawn(move || {
        let mut futures:HashMap<String, DateTime<Utc>> = HashMap::new();        
        loop {
            match rx.recv(){
                Ok((fname, level, info)) => {
                    if fname.contains("exit")|| fname.contains("quit"){
                        println!("exit cunder from signal");
                        break;
                    }else{
                        if futures.contains_key(&fname){
                            if let Some(v) = futures.get_mut(&fname){
                                if Utc::now() - *v > chrono::Duration::seconds( warning_interval ){    
                                    let color = match level {
                                        0 => White.normal(),
                                        1 => Cyan.normal(),
                                        2 => Green.normal(),
                                        3 => Blue.normal(),
                                        _ => Red.normal(),
                                    };

                                    let  now = Local::now().format("%d %H:%M:%S").to_string();                                       
                                    println!("{}: {}, {}",now,&fname, color.paint(&info));
                                    *v = Utc::now();
                                }
                            }
                        }else{
                            futures.insert(fname, Utc::now());
                        }
                    }
                }
                Err(er) => {
                    error!("recv error: {:?}", er);                    
                    break;
                }
            }
            //  thread::sleep(std::time::Duration::from_secs(1));
        }
    });


    event::run(ctx, events_loop, state);
}

fn tracelog_init(trace_level: &str) {
    let secs = chrono::Local::now().offset().local_minus_utc();
    let offset = UtcOffset::from_whole_seconds(secs).unwrap();

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(trace_level));
    let format = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]";

    // 写入控制台 stderr
    let formatting_layer = fmt::layer()
        .pretty()
        .with_writer(std::io::stderr)
        .with_timer(OffsetTime::new(
            offset,
            format_description::parse(format).unwrap(),
        ));
    // 注册
    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .init();
}

