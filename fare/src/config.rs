/*!
程序风格和阈值参数配置信息。

*/

// use std::collections::HashMap;
use serde::{Deserialize, Serialize};
type Result<T> = anyhow::Result<T>;

#[derive(Deserialize, Default, Debug, Serialize, Clone)]
pub struct Config {   
    #[serde(default)]
    /** 风格，true：深色，false：浅色 */
    pub dark_style : bool,    
    /** 图形标记的TOP数量， 过多容易乱 */
    pub top_count : usize,
    /** 距离均线的警告值，用于画线警示，或提醒交易 */
    pub warn_value : f32,
    /** 均值参数，部分商品均值是package总均值，需要除以参数, 默认为1 */
    pub average_package : i32,
}
/*
pub fn default_string() -> String {
    "".to_string()
}
*/

impl Config {
    /**
     * 生成样板配置并保存为config.toml
     *
     */
    #[allow(dead_code)]
    pub fn new_default(cfg: &str) -> Result<Self> {
        let src = r##"{                   
            "dark_style": true,
            "warn_value":42.0,
            "top_count":3,
            "average_package":1
        }"##;
        std::fs::write(cfg, src)?;
        let config: Config = serde_json::from_str(src)?;
        Ok(config)
    }

    /**
     * 根据config.toml生成Config对象。
     *
     */
    pub fn load(cfg: &str) -> Result<Self> {
        let src = String::from_utf8(std::fs::read(cfg)?)?;
        let config: Config = serde_json::from_str(src.as_str())?;
        /*
        if config.top_count == 0 {
            config.top_count = 1;
        }
        if config.warn_value < 0.1 {
            config.warn_value = 10.0;
        }
        if config.average_package < 1 {
            config.average_package = 1;
        } */
        Ok(config)
    }
}



#[derive(Debug, Clone, Default)]
pub struct SimpleTick {
    pub last_price: f32,
    pub average_price: f32,
    pub buy: i32,  // 提请交易的数量Buy
    pub sell: i32, // 提请交易的数量Sell
    pub update_time: String,
    pub volume: i32   
}

#[test]
fn abc_config_load_test() {
    let cfg_file = "./abc.json";
    let cfg = Config::new_default(cfg_file);
    match cfg {
        Ok(cfg) => {
            println!("{:#?}", cfg);
        }
        Err(e) => {
            println!("{}", e);
        }
    }

    let cfg2 = Config::load(cfg_file);
    match cfg2 {
        Ok(cfg) => {
            println!("{:#?}", cfg);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

#[test]
fn config_hash_test() {
    let cfg = Config {    
        dark_style: true,
        top_count: 3,
        warn_value: 42.0,
        average_package: 1,
    };
    println!("{:#?}", cfg);
    let sret = serde_json::to_string(&cfg).unwrap();
    println!("{}", sret);
}
