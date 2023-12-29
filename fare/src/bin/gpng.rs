/*!
基于plotters + egui + ebackend 的图表生成浏览器。
- 不仅支持图表的生成，还支持直接显示（基于egui）。
- 是plotters + egui + eframe的组合案例。

*/

use plotters::{prelude::*, style::full_palette::YELLOW};
use anyhow::{bail, Result};
use chrono::prelude::*;
use clap::Parser;
use ebackend::EguiBackend;
use eframe::egui::{self, CentralPanel, Visuals};
use egui::Key;
use std::path::Path;

// type Result<T> = anyhow::Result<T>;

#[derive(Parser, Debug)]
#[clap(
    name = "dunder",
    version = "1.0",
    author = "lengss",
    about = "futures assistant with egui and plotters"
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
    pub buy: i32,  // 提请交易的数量Buy
    pub sell: i32, // 提请交易的数量Sell
    pub update_time: String,
    pub volume: i32,
    pub radius: f32,
    pub sign: i32,
}

// const POWF: f32 = 0.2;
struct Gpng {
    futures: Vec<SimpleTick>,
    counter:i32,
    counter2:i32,
}

impl Gpng {
    fn new(cc: &eframe::CreationContext<'_>, fname: &str) -> Self {
        // Disable feathering as it causes artifacts
        let context = &cc.egui_ctx;

        context.tessellation_options_mut(|tess_options| {
            tess_options.feathering = false;
        });
        // Also enable light mode
        context.set_visuals(Visuals::light()); //dark());
        let mut futures: Vec<SimpleTick> = vec![];
        read_cvs(fname, &mut futures).unwrap();
        Gpng { futures, counter:0, counter2:0 }
    }

    fn defualt_lines(&self, ui: &mut egui::Ui ){        
        let root_area = EguiBackend::new(ui).into_drawing_area();
        root_area.fill(&WHITE).expect("fill erea failed");
        let root_area = root_area.titled("Image Title", ("sans-serif", 60)).unwrap();
        let (upper, lower) = root_area.split_vertically(512);
        let x_axis = (-3.4f32..3.4).step(0.1);

        let mut cc = ChartBuilder::on(&upper)
            .margin(5)
            .set_all_label_area_size(50)
            .caption("Sine and Cosine", ("sans-serif", 40))
            .build_cartesian_2d(-3.4f32..3.4, -1.2f32..1.2f32).unwrap();
      
        cc.configure_mesh()
            .x_labels(20)
            .y_labels(10)
            .disable_mesh()
            .x_label_formatter(&|v| format!("{:.1}", v))
            .y_label_formatter(&|v| format!("{:.1}", v))
            .draw().expect("configuring mesh failed");

        cc.draw_series(LineSeries::new(x_axis.values().map(|x| (x, x.sin())), &RED)).unwrap()
            .label("Sine")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        cc.draw_series(LineSeries::new(
            x_axis.values().map(|x| (x, x.cos())),
            &BLUE,
            )).unwrap()
            .label("Cosine")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

        cc.configure_series_labels().border_style(BLACK).draw().expect("drawing series labels failed");

/*
// It's possible to use a existing pointing element
 cc.draw_series(PointSeries::<_, _, Circle<_>>::new(
    (-3.0f32..2.1f32).step(1.0).values().map(|x| (x, x.sin())),
    5,
    Into::<ShapeStyle>::into(&RGBColor(255,0,0)).filled(),
))?;*/

        let cir: Vec<i32> = vec![1,5, 10, 20];
// Otherwise you can use a function to construct your pointing element yourself
        cc.draw_series(PointSeries::of_element(
            (1.0..3.0f32).step(1.0).values().map(|x| ( cir[x as usize] as f32, x*2.0)),
            15,
            ShapeStyle::from(&RED).filled(),
            &|coord, size, style| {
                EmptyElement::at(coord)
                + Circle::new((0, 0), size, style)
                + Text::new(format!("{:?}", coord), (0, 15), ("sans-serif", 15))
            },
        )).expect("draw series failed");

        let drawing_areas = lower.split_evenly((1, 2));

        for (drawing_area, idx) in drawing_areas.iter().zip(1..) {
            let mut cc = ChartBuilder::on(drawing_area)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .margin_right(20)
                .caption(format!("y = x^{}", 1 + 2 * idx), ("sans-serif", 40))
                .build_cartesian_2d(-1f32..1f32, -1f32..1f32).unwrap();
    
        cc.configure_mesh()
            .x_labels(5)
            .y_labels(3)
            .max_light_lines(4)
            .draw().expect("configure mesh failed");

        cc.draw_series(LineSeries::new(
            (-1f32..1f32)
            .step(0.01)
            .values()
            .map(|x| (x, x.powf(idx as f32 * 2.0 + 1.0))),
            &BLUE,
            )).expect("draw series failed");
    }
    // To avoid the IO failure being ignored silently, we manually call the present function
    root_area.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    }
   
}

impl eframe::App for Gpng {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {          

            self.counter += 1;
            if self.counter % 100 == 1 {                
                println!("检测 UI draw 100次消耗的时间： {}, {}",self.counter, Utc::now().format("%d/%m/%Y %H:%M:%S").to_string() );
            }
    
            if self.futures.len() == 0 {
                self.defualt_lines(ui);
                return;
            }

            let (max_v, min_v, volume, buy, sell, bs, vbs ) = get_serials(&self.futures, 5);
   
            let root_area = EguiBackend::new(ui).into_drawing_area();
            root_area.fill(&WHITE).expect("fill erea failed");
            let root_area = root_area.titled("Futrures plotter", ("sans-serif", 48)).unwrap();
            
            let x_axis = (0..self.futures.len()).step(1);
          
            let mut cc = ChartBuilder::on(&root_area)
                .margin(5)
                .set_all_label_area_size(50)
               // .caption("Sine and Cosine", ("sans-serif", 40))
                .build_cartesian_2d(0..self.futures.len(), min_v..max_v).unwrap();

            cc.configure_mesh()
                .x_labels(20)
                .y_labels(10)
                .disable_mesh()
                .x_label_formatter(&|v| format!("{}",  if *v < self.futures.len() { self.futures[*v].update_time.as_str() } else {"end"}  ))
                .y_label_formatter(&|v| format!("{:.1}", v))
                .draw().expect("configuring mesh failed");

            cc.draw_series(LineSeries::new(x_axis.values().map(|x| (x, self.futures[x].last_price)), &BLACK)).unwrap()
                .label("Price")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK));
   
            cc.draw_series(LineSeries::new(x_axis.values().map(|x| (x, self.futures[x].average_price )),&YELLOW,)).unwrap()
                .label("Average")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], YELLOW));

            // let tip_font_style = TextStyle::from(("sans-serif", 15).into_font()).color(&RED);
            // buy
            cc.draw_series(PointSeries::of_element(
                (0..buy.len()).step(1).values().map(|x| (buy[x].1, self.futures[ buy[x].1 ].last_price)),
                8,
                ShapeStyle::from(&BLUE.mix(0.6)).filled(),
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size, style)
                    // + Text::new(format!("{:?}", futures[coord.0].buy ), (0, 15), ("sans-serif", 15))
                },
            )).unwrap()
            .label("Buy")
            .legend(|(x, y)|  Circle::new((x+10, y), 5, BLUE));
                
            // sell
            cc.draw_series(PointSeries::of_element(
                (0..buy.len()).step(1).values().map(|x| (sell[x].1, self.futures[ sell[x].1 ].last_price)),
                8,
                ShapeStyle::from(&GREEN.mix(0.6)).filled(),
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size, style)
                    // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
                },
            )).unwrap()
            .label("Sell")
            .legend(|(x, y)| Circle::new((x+10, y), 5, GREEN));
            // bs
            cc.draw_series(PointSeries::of_element(
                (0..bs.len()).step(1).values().map(|x| (bs[x].1, self.futures[ bs[x].1 ].last_price)),
                12,
                ShapeStyle::from(&RED.mix(0.4)).filled(),
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        +  Rectangle::new([(0, 0),(size, size)], style)
                    // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
                },
            )).unwrap()
            .label("B+S")
            .legend(|(x, y)| Rectangle::new([(x+5, y-5),(x+15, y+5)], RED));

            // volume
            cc.draw_series(PointSeries::of_element(
                (0..volume.len()).step(1).values().map(|x| (volume[x].1, self.futures[ volume[x].1 ].last_price)),
                12,
                ShapeStyle::from(&GREEN.mix(0.4)).filled(),
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        +   TriangleMarker::new((0, 0), size, style)
                    // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
                },
            )).unwrap()
            .label("Volume")
            .legend(|(x, y)| TriangleMarker::new((x+10, y), 6, GREEN));

            // V/a
            cc.draw_series(PointSeries::of_element(
                (0..vbs.len()).step(1).values().map(|x| (vbs[x].1, self.futures[ vbs[x].1 ].last_price)),
                12,
                ShapeStyle::from(&RED.mix(0.4)).filled(),
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        +   TriangleMarker::new((0, 0), size, style)
                    // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
                },
            )).unwrap()
            .label("V/a")
            .legend(|(x, y)| TriangleMarker::new((x+10, y), 6, RED));
        

            let mv1 =  max_v - self.futures[0].average_price.max(self.futures[0].last_price);
            let mv2 =  self.futures[0].average_price.min(self.futures[0].last_price) - min_v;
            let pos =  if mv1 > mv2 {  
                SeriesLabelPosition::UpperLeft
            } else {
                SeriesLabelPosition::LowerLeft
            };    
            cc.configure_series_labels().position(pos ) .border_style(BLACK).draw().expect("drawing series labels failed");

            // To avoid the IO failure being ignored silently, we manually call the present function
            root_area.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
  
        });
        // println!("Result has been saved to {}", OUT_FILE_NAME);
        if ctx.input(|input| input.key_pressed(Key::F1)) {
            println!("F1 pressed");
        }
        
        self.counter2 += 1;
        if self.counter2 % 1000 == 1 {                
            println!("检测 update 1000次消耗的时间： {}, {}",self.counter2, Utc::now().format("%d/%m/%Y %H:%M:%S").to_string() );
        }
       // 这个逻辑对于静态数据，是没有必要的，只会增加CPU消耗。屏幕刷新事件和键盘鼠标事件，都会自动触发 update.
       // std::thread::sleep(Duration::from_millis(100));
       // ctx.request_repaint();
            
       /*
       // 这个逻辑行不通， 没有ctx.request_repaint()，就不会触发update
       if self.counter % 100 == 1 { 
            ctx.request_repaint();
            println!("重新repaint");
        }
       */
      /* 
      // 这个逻辑也行不通， 没有ctx.request_repaint()，就不会触发update，也就不会执行read_message()
      
      if self.read_message() {
        ctx.request_repaint();
        }
    */

    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    if !Path::new(&cli.futures).exists() {
        println!("futures file(.csv) is not exists");
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 840.0]),
        ..Default::default()
    };

    let fname  = Box::leak(cli.futures.into_boxed_str());

    eframe::run_native(
        "Hello, Fare (futures assistent with rust egui)",
        native_options,
        Box::new(|cc| Box::new(Gpng::new(cc, fname))),
    )
    .unwrap();
    Ok(())
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

