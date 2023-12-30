/*!
各种期货商品的阈值配置信息。

*/

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
type Result<T> = anyhow::Result<T>;

#[derive(Deserialize, Default, Debug, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    /** udp server port for ticks*/
    pub udp_port: u16,
    #[serde(default)]
    /** windows 高  */
    pub height: f32,
    #[serde(default)]
    /** bar对应像素数，也即两个数据间的距离 */
    pub bar_size: f32,

    #[serde(default)]
    /** font size */
    pub font_size: f32,

    #[serde(default)]
    /** 价格变化加速度报警阈值 */
    pub av_max: f32,
    #[serde(default)]
    /** 商品对应的 Buy+Sell报警阈值 */
    pub bs_max: HashMap<String, i32>,
    #[serde(default)]
    /** average 参数，有些商品的average是多单和，需要计算，除以参数 */
    pub avargs: HashMap<String, f32>,
    /** 各商品的偏离均值报警阈值 */
    pub warnings: HashMap<String, f32>,
    /** 相同商品告警最小间隔（单位：秒） */
    pub warning_interval: i64,
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
    pub fn new_default(cfg: &str) -> Result<Self> {
        let src = r##"{          
            "udp_port":12346,
            "height": 1600.0,    
            "font_size": 26.0,    
            "av_max":15.0,        
            "warning_interval": 50,         
            "bs_max":{
                "rb1810": 2000,
                "AP401":5000
            },
            "avargs":{
                "rb1810": 5.0,
                "AP401":1.0
            },
            "warnings":{
                "rb1810": 40.0,
                "AP401":35.0
            },
            "bar_size": 2.0  
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
        Ok(config)
    }
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
    let mut cfg = Config {
        udp_port: 10,
        height: 20.0,
        bar_size: 2.0,
        av_max: 15.0,
        bs_max: HashMap::new(),
        font_size: 26.0,
        warning_interval:54,
        avargs: HashMap::new(),
        warnings: HashMap::new(),
    };
    cfg.avargs.insert("rb1810".to_string(), 5.0);
    cfg.avargs.insert("rb1812".to_string(), 5.0);

    println!("{:#?}", cfg);
    let sret = serde_json::to_string(&cfg).unwrap();
    println!("{}", sret);
}
