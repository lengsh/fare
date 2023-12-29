/*!
基于plotters的png图表生成器。
- 优势是干净，依赖包少；
- 缺点是不支持图表的显示，尤其是动态显示。

*/
use plotters::{prelude::*, style::full_palette::YELLOW};
// use csv::{Reader, ReaderBuilder, Writer, WriterBuilder};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{bail, Result};
use clap::Parser;
// type Result<T> = anyhow::Result<T>;

#[derive(Parser, Debug)]
#[clap(
    name = "dunder",
    version = "1.0",
    author = "lengss",
    about = "futures assistant"
)]
pub struct Cli {
    /// Sets a custom trace filter, such as info|debug|error|...
    #[clap(name = "trace", short('e'), long, default_value = "error")]
    trace: String,
    #[clap(name = "file", short, long, default_value = "")]
    futures: String,    
    #[clap(name = "output", short, long, default_value = "")]
    output: String,    
}

#[derive(Debug, Clone, Default)]
pub struct SimpleTick {
    pub last_price: f32,
    pub average_price: f32,
    pub max: f32,
    pub min: f32,
    pub buy: i32, // 提请交易的数量Buy
    pub sell: i32, // 提请交易的数量Sell
    pub update_time: String,
    pub volume: i32,
    pub radius: f32,
    pub sign: i32,
}

//const POWF: f32 = 0.2;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let cli = Cli::parse();
    if !Path::new(&cli.futures).exists() {
        println!("futures file(.csv) is not exists");        
        return Ok(());
    }

    if cli.output.len() > 0 && !Path::new(&cli.output).exists() {
        fs::create_dir_all(&cli.output)?;              
    }

    let mut futures:Vec<SimpleTick> = vec![];
    read_cvs( &cli.futures , &mut futures)?;
    if futures.len() < 1 {
        println!("futures.len() < 1");
        return Err(From::from("futures.len() < 1"));
    }
    println!("count = {}", futures.len());
    let f_name = Path::new(&cli.futures).file_name().unwrap().to_str().unwrap();
    let png_name = f_name.replace("csv", "png");
    
    let (max_v, min_v, volume, buy, sell, bs, vbs ) = get_serials(&futures, 5);
   
    let png_file = PathBuf::from(cli.output).join(png_name);
    let root_area = BitMapBackend::new(&png_file, (2800, 768)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let root_area = root_area.titled("Futures Analysis", ("sans-serif", 60))?;
    let x_axis = (0..futures.len()).step(1);
    // root_area.draw_text(&max_result, &TextStyle::from(("sans-serif", 30).into_font()).color(&BLACK), (500, 100))?;

    let mut cc = ChartBuilder::on(&root_area)
        .margin(5)
        .set_all_label_area_size(50)
        .caption(f_name, ("sans-serif", 40))
        .build_cartesian_2d(0..futures.len(), min_v..max_v)?;

    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{}",  if *v < futures.len() {futures[*v].update_time.as_str() } else {"end"}  ))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()?;

    cc.draw_series(LineSeries::new(x_axis.values().map(|x| (x, futures[x].last_price)), &BLACK))?
        .label("Price")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK));

    cc.draw_series(LineSeries::new(
        x_axis.values().map(|x| (x, futures[x].average_price )),
        &YELLOW,
    ))?
    .label("Average")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], YELLOW));

   
    // buy
    cc.draw_series(PointSeries::of_element(
        (0..buy.len()).step(1).values().map(|x| (buy[x].1, futures[ buy[x].1 ].last_price)),
        8,
        ShapeStyle::from(&BLUE.mix(0.6)).filled(),
        &|coord, size, style| {
            EmptyElement::at(coord)
                + Circle::new((0, 0), size, style)
               // + Text::new(format!("{:?}", futures[coord.0].buy ), (0, 15), ("sans-serif", 15))
        },
    ))?
    .label("Buy")
    .legend(|(x, y)|  Circle::new((x+10, y), 5, BLUE));
        
    // sell
    cc.draw_series(PointSeries::of_element(
        (0..buy.len()).step(1).values().map(|x| (sell[x].1, futures[ sell[x].1 ].last_price)),
        8,
        ShapeStyle::from(&GREEN.mix(0.6)).filled(),
        &|coord, size, style| {
            EmptyElement::at(coord)
                + Circle::new((0, 0), size, style)
               // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
        },
    ))?
    .label("Sell")
    .legend(|(x, y)| Circle::new((x+10, y), 5, GREEN));
    // bs
    cc.draw_series(PointSeries::of_element(
        (0..bs.len()).step(1).values().map(|x| (bs[x].1, futures[ bs[x].1 ].last_price)),
        12,
        ShapeStyle::from(&RED.mix(0.4)).filled(),
        &|coord, size, style| {
            EmptyElement::at(coord)
                +  Rectangle::new([(0, 0),(size, size)], style)
            // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
        },
    ))?
    .label("B+S")
    .legend(|(x, y)| Rectangle::new([(x+5, y-5),(x+15, y+5)], RED));

    // volume
    cc.draw_series(PointSeries::of_element(
        (0..volume.len()).step(1).values().map(|x| (volume[x].1, futures[ volume[x].1 ].last_price)),
        12,
        ShapeStyle::from(&GREEN.mix(0.4)).filled(),
        &|coord, size, style| {
            EmptyElement::at(coord)
                +   TriangleMarker::new((0, 0), size, style)
            // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
        },
    ))?
    .label("Volume")
    .legend(|(x, y)| TriangleMarker::new((x+10, y), 6, GREEN));

    // V/a
    cc.draw_series(PointSeries::of_element(
        (0..vbs.len()).step(1).values().map(|x| (vbs[x].1, futures[ vbs[x].1 ].last_price)),
        12,
        ShapeStyle::from(&RED.mix(0.4)).filled(),
        &|coord, size, style| {
            EmptyElement::at(coord)
                +   TriangleMarker::new((0, 0), size, style)
            // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
        },
    ))?
    .label("V/a")
    .legend(|(x, y)| TriangleMarker::new((x+10, y), 6, RED));

           
    cc.configure_series_labels().border_style(BLACK).draw()?;
  
    // To avoid the IO failure being ignored silently, we manually call the present function
    root_area.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {:?}", png_file);
    Ok(())
}


    /**
    生成top n条数据, 包括TOP volume，TOP buy, TOP sell, TOP V/a
    */
pub fn get_serials(futures: &Vec<SimpleTick>, top_size: usize) -> (f32, f32, Vec<(i32, usize)>, Vec<(i32, usize)>, Vec<(i32, usize)>, Vec<(i32, usize)>, Vec<(f32, usize)>) {
        let len = futures.len();
        if len == 0 {
            println!("No data");
            return (0.0, 0.0, vec![], vec![], vec![], vec![], vec![]);   
        }
            
        let mut lastv = futures[0].volume;        
        let mut volume: Vec<(i32,usize)> = vec![];
        let mut buy: Vec<(i32,usize)> = vec![];
        let mut sell: Vec<(i32,usize)> = vec![];
        let mut bs: Vec<(i32,usize)> = vec![];
        let mut vbs: Vec<(f32, usize)> = vec![];
        let mut maxv: Vec<(f32,usize)> = vec![];
        let mut minv: Vec<(f32, usize)> = vec![];

        for i in 0..futures.len() {
            if i == 0 {
                volume.push((0, 0));
                buy.push((futures[0].buy, 0));
                sell.push((futures[0].sell, 0));
                bs.push( (futures[0].sell + futures[0].buy , 0)  );
                vbs.push((0.0, 0));
                maxv.push((futures[0].last_price, 0));
                minv.push((futures[0].last_price, 0));
            } else {
                let vol = futures[i].volume - lastv; 
                if vol > volume[ volume.len()-1].0 {
                    volume.push( (vol, i))    
                }                
                lastv = futures[i].volume;

                if futures[i].buy > buy[buy.len()-1].0 {
                    buy.push((futures[i].buy, i))
                }
                if futures[i].sell > sell[ sell.len() - 1].0 {
                    sell.push((futures[i].sell, i))
                }
                let  bos = futures[i].sell + futures[i].buy;
                if  bos  > bs[ bs.len() - 1].0 {
                    bs.push((futures[i].sell + futures[i].buy , i))
                }
                let vbosv = if bos > 10 { vol as f32 / bos as f32   } else {  0.0 };
                if vbosv > vbs[  vbs.len() - 1].0 {
                    vbs.push((vbosv, i))
                }

                if futures[i].last_price > maxv[ maxv.len() - 1].0 {
                    maxv.push((futures[i].last_price, i))
                } else if futures[i].last_price < minv[ minv.len() - 1].0 {
                    minv.push((futures[i].last_price, i))
                } 

                //  
                volume.sort_by(|a,b | b.0.cmp(&a.0) );
                if volume.len() > top_size {
                    volume.remove(volume.len()-1);
                }

                buy.sort_by(|a,b | b.0.cmp(&a.0) );
                if buy.len() > top_size {
                    buy.remove(buy.len()-1);
                }

                sell.sort_by(|a,b | b.0.cmp(&a.0) );
                if sell.len() > top_size {
                    sell.remove(sell.len()-1);
                }

                bs.sort_by(|a,b | b.0.cmp(&a.0) );
                if bs.len() > top_size {
                    bs.remove(bs.len()-1);
                }
                
                vbs.sort_by(|a,b | b.0.partial_cmp(&a.0).unwrap() );
                if vbs.len() > top_size {
                    vbs.remove(vbs.len()-1);
                }

                maxv.sort_by(|a,b | b.0.partial_cmp(&a.0).unwrap() );
                if maxv.len() > top_size {
                    maxv.remove(maxv.len()-1);
                }

                minv.sort_by(|a,b | a.0.partial_cmp(&b.0).unwrap() );
                if minv.len() > top_size {
                    minv.remove(minv.len()-1);
                }
            }
        }    
        return (maxv[0].0, minv[0].0, volume, buy, sell, bs, vbs)      
    }

/*
从csv文件中读取数据，并保存到futures中，
格式为：update_time,last_price, max, min, updown, average_price, volume, radius,
*/
pub fn read_cvs(fname: &str, futures: &mut Vec<SimpleTick>) -> Result<()> {
    if !Path::new(fname).exists() {
        bail!("file not found");
    }

    let mut rdr = csv::ReaderBuilder::new().from_path(fname)?;
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
        futures.push(tick);  
    }
    Ok(())
}
