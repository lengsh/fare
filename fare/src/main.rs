/*!
基于plotters + egui的期货交易行情分析助手。
- 借助plotters实现线性图表的生成；
- 借助egui实现图表的动态显示；
- 通过eframe + ebackend, 实现从plotters到egui的输出。
- 采用csv文件保存行情数据。
- 借助tokio实现异步行情数据接收和加工。

*/
use anyhow::{bail, Result};
use chrono::prelude::*;
use clap::Parser;
use ebackend::EguiBackend;
use eframe::egui::{self, CentralPanel, Visuals};
use egui::Key;
use plotters::prelude::*;
use plotters::chart::SeriesLabelPosition;
use time::{format_description, UtcOffset};
use tracing::{debug, error};
use tracing_subscriber::{
    filter::EnvFilter, fmt, fmt::time::OffsetTime, layer::SubscriberExt, util::SubscriberInitExt,
    Registry,
};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast ;
mod config;
mod shared;
mod worker;

pub(crate) use config::{Config, SimpleTick};
pub(crate) use shared::Shared;
pub(crate) use worker::*;

const DEFAULT_PORT: u16 = 12346;

#[derive(Parser, Debug)]
#[clap(
    name = "fare",
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
    #[clap(short, long, default_value_t = DEFAULT_PORT)]
    port: u16,
    #[clap(name = "cfg", short, long, default_value = "config.json")]
    cfg: String,
    //#[clap(name = "output", short, long, default_value = "")]
    //output: String,
}
// background, border, price, average, font
fn get_color_style(dark: bool) -> (RGBColor, RGBColor, RGBColor, RGBColor, RGBColor) {
    if dark {
        (
            BLACK,
            RGBColor(180, 180, 180),
            RGBColor(180, 180, 180),
            YELLOW,
            RGBColor(180, 180, 180),
        )
    } else {
        (WHITE, BLACK, BLACK, YELLOW, BLACK)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    tracelog_init(&cli.trace);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 560.0]),
        ..Default::default()
    };

    let fcsv = Box::leak(cli.futures.into_boxed_str());
    let fconfig = Box::leak(cli.cfg.into_boxed_str());
   // let fout = Box::leak(datapath.into_boxed_str());
    let addr = Box::leak(format!("0.0.0.0:{}", cli.port).into_boxed_str());
    eframe::run_native(
        "Hello, Fare (futures assistant with rust egui)",
        native_options,
        Box::new(|cc| Box::new(Fare::new(cc, fconfig, fcsv, addr))),
    )
    .unwrap();

    Ok(())
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

struct Fare {   
   // notify_shutdown: broadcast::Sender<()>,
    sync_chan: broadcast::Sender<String>,
    shared: Arc<Shared>,
}

impl Fare {
    fn new(
        cc: &eframe::CreationContext<'_>,
        fconfig: &str,
        fcsv: &str,    
        addr: &str,
    ) -> Self {
        // 创建 shared handle对象
        let mut shared = Shared::new(fconfig, addr);
        if Path::new(fcsv).exists() {
            shared.read_cvs(fcsv).unwrap_or_default();
        }

        // Disable feathering as it causes artifacts
        let context = &cc.egui_ctx;
        context.tessellation_options_mut(|tess_options| {
            tess_options.feathering = false;
        });
        // Also enable light mode
        let mode_dark = shared.config.dark_style;
        if mode_dark {
            context.set_visuals(Visuals::dark());
        } else {
            context.set_visuals(Visuals::light());
        }

        // 创建 shared handle对象

        let shared = Arc::new(shared);
        // let (notify_shutdown, shutdown) = broadcast::channel(1);
        let (sync_chan, cmd) = broadcast::channel(1);
        let mut worker = Worker::new(Arc::clone(&shared), /*  shutdown, */ cmd, cc.egui_ctx.clone());

        let _app = tokio::spawn(async move {
            if let Err(err) = worker.run().await {
                error!("{} {}", err, "connection error");
            } else {
            }
        });

        Fare {
            shared,
           // notify_shutdown,
            sync_chan,
        }
    }

    fn defualt_example(&self, ui: &mut egui::Ui) {
        let root_area = EguiBackend::new(ui).into_drawing_area();
        root_area.fill(&WHITE).expect("fill erea failed");
        let root_area = root_area.titled("Welcome to fare", ("sans-serif", 60)).unwrap();
        // let (upper, lower) = root_area.split_vertically(512);
        let x_axis = (-3.4f32..3.4).step(0.1);

        let title_style = TextStyle::from(("sans-serif", 40).into_font()).color(&RED);

        let mut cc = ChartBuilder::on(&root_area)
            .margin(5)
            .set_all_label_area_size(50)
            .caption("No signal now! waiting for ...", title_style.clone() )
            .build_cartesian_2d(-3.4f32..3.4, -1.2f32..1.2f32)
            .unwrap();

        cc.configure_mesh()
            .x_labels(20)
            .y_labels(10)
            .disable_mesh()
            .x_label_formatter(&|v| format!("{:.1}", v))
            .y_label_formatter(&|v| format!("{:.1}", v))
            .draw()
            .expect("configuring mesh failed");

        cc.draw_series(LineSeries::new(x_axis.values().map(|x| (x, x.sin())), &RED))
            .unwrap()
            .label("Sine")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        cc.draw_series(LineSeries::new(
            x_axis.values().map(|x| (x, x.cos())),
            &BLUE,
        ))
        .unwrap()
        .label("Cosine")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

        cc.configure_series_labels()
            .border_style(BLACK)
            .draw()
            .expect("drawing series labels failed");

        cc.draw_series(PointSeries::of_element(
            (-3.0f32..2.1f32).step(1.0).values().map(|x| (x, x.sin())),
            5,
            ShapeStyle::from(&RED).filled(),
            &|coord, size, style| {
                EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                    + Text::new(format!("{:?}", coord), (0, 15), ("sans-serif", 15))
            },
        ))
        .expect("draw series failed");
        /* 
        let drawing_areas = lower.split_evenly((1, 2));

        for (drawing_area, idx) in drawing_areas.iter().zip(1..) {
            let mut cc = ChartBuilder::on(drawing_area)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .margin_right(20)
                .caption(format!("y = x^{}", 1 + 2 * idx), ("sans-serif", 40))
                .build_cartesian_2d(-1f32..1f32, -1f32..1f32)
                .unwrap();

            cc.configure_mesh()
                .x_labels(5)
                .y_labels(3)
              //  .max_light_lines(4)
                .draw()
                .expect("configure mesh failed");

            cc.draw_series(LineSeries::new(
                (-1f32..1f32)
                    .step(0.01)
                    .values()
                    .map(|x| (x, x.powf(idx as f32 * 2.0 + 1.0))),
                &BLUE,
            ))
            .expect("draw series failed");
        } */
        // To avoid the IO failure being ignored silently, we manually call the present function
        root_area.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    }
}

impl eframe::App for Fare {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        println!("on exit");
        // self.notify_shutdown.send(()).unwrap_or_default();
        self.sync_chan.send("exit".to_owned()).unwrap_or_default();
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            if self.shared.get_size() == 0 {
                self.defualt_example(ui);
                return;
            }

            // 以下的逻辑，都是基于futures的数量 > 0 的时候，才会执行
            let title = self.shared.get_futures_name();
            let title = if title.len() > 0 {
                title
            } else {
                "Futures assistant".to_owned()
            };

            let (bk, bd, cp, ca, cf) = get_color_style(self.shared.config.dark_style);
            let top_count = self.shared.config.top_count;
            let (f_max, f_min, buy, sell, volume, bs, vbs) = self.shared.get_topx(top_count);
         
            let root_area = EguiBackend::new(ui).into_drawing_area();
            root_area.fill(&bk).expect("fill erea failed");

            let title_style = TextStyle::from(("sans-serif", 40).into_font()).color(&RED);
            let original_style = ShapeStyle {
                color: bd.mix(0.6),
                filled: true,
                stroke_width: 1,
            };

            let root_area = root_area.titled(&title, title_style).unwrap();

            let futures = &(*(self.shared.futures.data.read().unwrap()));
            let len = futures.len();
            let (max_v, min_v) = if f_max.len() > 0 {  // 此时 len肯定大于0； 把均值也放入大小比较范围！，第一个均值即可。
                (f_max[0].0.max(futures[0].average_price ), f_min[0].0.min(futures[0].average_price))
            } else {
                (1.0, 0.0)
            };
            let x_axis = (0..len).step(1);

            let mut cc = ChartBuilder::on(&root_area)
                .margin(5)
                .set_all_label_area_size(50)
                // .caption("Futures assistant", ("sans-serif", 40))
                .build_cartesian_2d(0..len, min_v..max_v)
                .unwrap();

            let label_style = TextStyle::from(("sans-serif", 12).into_font()).color(&cf);
            cc.configure_mesh()
                .x_labels(20)
                .label_style(label_style.clone())
                .y_labels(10)
                .disable_mesh()
                .x_label_formatter(&|v| {
                    format!(
                        "{}",
                        if *v < len {
                            futures[*v].update_time.as_str()
                        } else {
                            "end"
                        }
                    )
                })
                .y_label_formatter(&|v| format!("{:.1}", v))
                .axis_style(original_style)
                .draw()
                .expect("configuring mesh failed");

            cc.draw_series(LineSeries::new(
                x_axis.values().map(|x| (x, futures[x].last_price)),
                &cp,
            ))
            .unwrap()
            .label("Price")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 30, y)], cp));

            cc.draw_series(LineSeries::new(
                x_axis.values().map(|x| (x, futures[x].average_price)),
                &ca.mix(0.5),
            ))
            .unwrap()
            .label("Average")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 30, y)], ca));

            // 均线上下增加2条价格警示线，在max_v和min_v范围内则显示，否则不予显示。            
            let warn_val = self.shared.config.warn_value;
            let warn_label = format!("Warn:{:.1}", warn_val);
            cc.draw_series(LineSeries::new(
                x_axis.values().filter(|x| futures[*x].average_price + warn_val <= max_v ).map(|x| (x, futures[x].average_price + warn_val )),
                &RGBColor(160, 10, 10).mix(0.3),
            ))
            .unwrap()
            .label(&warn_label)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 30, y)], RGBColor(160, 10, 10)));

            cc.draw_series(LineSeries::new(
                x_axis.values().filter(|x| futures[*x].average_price - warn_val >= min_v).map(|x| (x, futures[x].average_price - warn_val )),
                &RGBColor(160, 10, 10).mix(0.3),
            ))
            .unwrap();

            if len > 30 {
                // 过早显示意义不大！
                // buy
                cc.draw_series(PointSeries::of_element(
                    (0..buy.len())
                        .step(1)
                        .values()
                        .map(|x| (buy[x].1, futures[buy[x].1].last_price)),
                    8,
                    ShapeStyle::from(&BLUE.mix(0.6)).filled(),
                    &|coord, size, style| {
                        EmptyElement::at(coord) + Circle::new((0, 0), size, style)
                        // + Text::new(format!("{:?}", futures[coord.0].buy ), (0, 15), ("sans-serif", 15))
                    },
                ))
                .unwrap()
                .label("Buy")
                .legend(|(x, y)| Circle::new((x + 10, y), 5, BLUE));

                // sell
                cc.draw_series(PointSeries::of_element(
                    (0..buy.len())
                        .step(1)
                        .values()
                        .map(|x| (sell[x].1, futures[sell[x].1].last_price)),
                    8,
                    ShapeStyle::from(&GREEN.mix(0.6)).filled(),
                    &|coord, size, style| {
                        EmptyElement::at(coord) + Circle::new((0, 0), size, style)
                        // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
                    },
                ))
                .unwrap()
                .label("Sell")
                .legend(|(x, y)| Circle::new((x + 10, y), 5, GREEN));
                // bs
                cc.draw_series(PointSeries::of_element(
                    (0..bs.len())
                        .step(1)
                        .values()
                        .map(|x| (bs[x].1, futures[bs[x].1].last_price)),
                    12,
                    ShapeStyle::from(&RED.mix(0.4)).filled(),
                    &|coord, size, style| {
                        EmptyElement::at(coord) + Rectangle::new([(0, 0), (size, size)], style)
                        // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
                    },
                ))
                .unwrap()
                .label("B+S")
                .legend(|(x, y)| Rectangle::new([(x + 5, y - 5), (x + 15, y + 5)], RED));

                // volume
                cc.draw_series(PointSeries::of_element(
                    (0..volume.len())
                        .step(1)
                        .values()
                        .map(|x| (volume[x].1, futures[volume[x].1].last_price)),
                    12,
                    ShapeStyle::from(&GREEN.mix(0.4)).filled(),
                    &|coord, size, style| {
                        EmptyElement::at(coord) + TriangleMarker::new((0, 0), size, style)
                        // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
                    },
                ))
                .unwrap()
                .label("Volume")
                .legend(|(x, y)| TriangleMarker::new((x + 10, y), 6, GREEN));

                // V/a
                cc.draw_series(PointSeries::of_element(
                    (0..vbs.len())
                        .step(1)
                        .values()
                        .map(|x| (vbs[x].1, futures[vbs[x].1].last_price)),
                    12,
                    ShapeStyle::from(&RED.mix(0.4)).filled(),
                    &|coord, size, style| {
                        EmptyElement::at(coord) + TriangleMarker::new((0, 0), size, style)
                        // + Text::new(format!("{:?}", futures[coord.0].sell ), (0, 15), ("sans-serif", 15))
                    },
                ))
                .unwrap()
                .label("V/a")
                .legend(|(x, y)| TriangleMarker::new((x + 10, y), 6, RED));
            }
            /*  比较开始曲线的位置，定位较大的空白区间，然后根据位置，确定label的位置。   */        
            let pos = if max_v - futures[0].last_price > futures[0].last_price - min_v {
                SeriesLabelPosition::UpperLeft
            } else {
                SeriesLabelPosition::LowerLeft
            };
            cc.configure_series_labels()
                .position(pos)
                .border_style(bd)
                .label_font(label_style)
                .draw()
                .expect("drawing series labels failed");

            // To avoid the IO failure being ignored silently, we manually call the present function
            root_area
                .present()
                .expect("Unable to write, it's impporsable");
        });
        if ctx.input(|input| input.key_pressed(Key::F1)) {
            self.shared.print_config();
        } else if ctx.input(|input| input.key_pressed(Key::F2)) {
            self.shared.print_last_data(10);
        } else if ctx.input(|input| input.key_pressed(Key::F3)) {
            self.shared.print_tops(10);
        } else if ctx.input(|input| input.key_pressed(Key::F4)) {
            self.shared.report();
        } else if ctx.input(|input| input.key_pressed(Key::F5)) {
            self.sync_chan.send("test".to_owned()).unwrap_or_default();
        } else if ctx.input(|input| input.key_pressed(Key::Delete)) {
            self.sync_chan.send("clear".to_owned()).unwrap_or_default();
            /* 
            let futures_name = self.shared.get_futures_name();
            self.shared.write_cvs(&futures_name).unwrap();
            self.shared.clear();
            */
            error!("clear data");
        }
        /*

        let response = egui::Window::new("Window 1").show(egui_ctx, |ui| {
        ui.vertical(|ui| {
            ui.label("Label 1");
            ui.label("Label 2");
            ui.label("Label 3");
        });
        }).unwrap();
        if response.hovered() {
            println!("hovered!");
        }  
        */

    }
    
}
