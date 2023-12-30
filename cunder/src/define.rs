//! 常量定义和数据结构定义

/** 默认边界宽度 */
pub const BORDER_WIDTH: f32 = 20.0;
/** 窗口最小高度 */
pub const WIN_MIN_HEIGHT: f32 = 200.0;

/** 小圆形警示图的半径 */
pub const SIGN_RADIUS: f32 = 12.0;

/** 大圆形振幅偏离警告图的半径 */
pub const WARNING_RADIUS :f32 = 50.0;

/** 默认的价格偏离加速报警阈值，一分钟内变化量 */
pub const WARNING_AV_DEFUALT:f32 = 30.0;

/** 默认的行情数据索引文件名 */
pub const FUTURES_INDEX_NAME: &str = "futures_index.csv";

/**  foxy UPD发送数的字段个数 */
pub const FOXY_TICK_LEN:usize = 9;
/**
默认的价格偏离报警阈值，可以通过配置文件进行特定商品的定制
*/
pub const BUY_AND_SELL_WARNING: f32 = 50.0;
/**
申请买入和卖出的总量默认报警阈值，可以通过配置文件进行特定商品的定制
*/
pub const BUY_AND_SELL_DEFAULT: i32 = 20000;
/**
 * 
 */
pub const BS_TOP_AS_WARNING :usize = 9;
pub const BS_TOP_SIZE :usize = 10;
/**
从行情订阅转发foxy接收到的tick数据的本地化保存对象。

接收UDP数据依赖foxy,格式：
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
#[derive(Debug, Clone, Default)]
pub struct SimpleTick {
    /** 最新价 */
    pub last_price: f32,
    /** 平均价 */
    pub average_price: f32,
    /** 当前数据的价格变动最大价（1分钟内） */
    pub max: f32,
    /** 当前数据的价格变动最小价（1分钟内） */
    pub min: f32,
    /** 申请买入总量 */
    pub buy: i32, 
    /** 申请卖出总量 */
    pub sell: i32,
    /** 更新时间, 格式为 HH:MM */
    pub update_time: String,
    /** 成交量, 累计值 */
    pub volume: i32,
    /** 价格与均值的差值 */
    pub radius: f32,
    /** 标签显示信号，便于图形画标注某个点 */
    pub sign: i32,
}

#[test]
fn hello_lambda() {
    
    let mut values: Vec<SimpleTick> = vec![];
    let mut tick = SimpleTick::default();
    tick.sign = 10;
    values.push(tick);
    let mut t2 = SimpleTick::default();
    t2.sign = 20;
    values.push(t2);

    for i in 0..values.len() {
        println!("{:?}",&values[i]);
    }

    values.sort_by(|a, b| b.sign.cmp(&a.sign));

    
    for i in 0..values.len() {
        println!("{:?}",&values[i]);
    }
        
}
