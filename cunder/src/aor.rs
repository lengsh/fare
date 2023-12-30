//! Assessment of Risk，风险评估

use crate::define::SimpleTick;
use anyhow::Ok;
type Result<T> = anyhow::Result<T>;

pub struct Aor {
    pub name: String,
    pub up_max: f32,
    pub down_max: f32,
    pub waves: Vec<SimpleTick>,
}

impl Aor {
    pub fn new(name: String) -> Self {
        Aor {
            name,
            up_max: 0.0,
            down_max: 0.0,
            waves: vec![],
        }
    }
    /*
    pub fn clear(&mut self) {
        self.waves.clear();
    } */

    pub fn next(&mut self, ticks: &SimpleTick) -> Result<bool> {
        let dif = ticks.last_price - ticks.average_price;
        if self.waves.is_empty() {
            if dif != 0.0 {
                let mut tick = ticks.clone();
                tick.radius = dif;
                self.waves.push(tick);
                if dif > 0.0 {
                    self.up_max = dif;
                } else {
                    self.down_max = dif;
                }
            }
        } else {
            //
            if dif > 0.0 {
                // Up
                if dif > self.up_max {
                    self.up_max = dif;
                }
                let last = self.waves.len() - 1;
                if self.waves[last].radius < 0.0 {
                    // Down
                    let mut tick = ticks.clone();
                    tick.radius = dif;
                    self.waves.push(tick);
                } else {
                    if self.waves[last].radius < dif {
                        // more big
                        self.waves[last].last_price = ticks.last_price;
                        self.waves[last].average_price = ticks.average_price;
                        self.waves[last].update_time = ticks.update_time.clone();
                        self.waves[last].radius = dif;
                    }
                }
            }
            if dif < 0.0 {
                // Down
                if dif < self.down_max {
                    self.down_max = dif;
                }

                let last = self.waves.len() - 1;
                if self.waves[last].radius > 0.0 {
                    // Up
                    let mut tick = ticks.clone();
                    tick.radius = dif;
                    self.waves.push(tick);
                } else {
                    if self.waves[last].radius > dif {
                        // more small
                        self.waves[last].last_price = ticks.last_price;
                        self.waves[last].average_price = ticks.average_price;
                        self.waves[last].update_time = ticks.update_time.clone();
                        self.waves[last].radius = dif;
                    }
                }
            }
        }
        Ok(true)
    }
    /*
    pub fn print(&self) {
        println!("{} Assessment of Risk", self.name);
        for i in 0..self.waves.len() {
            println!("{} : {:#?}", i, self.waves[i]);
        }
        let mut u_val = 0.0;
        let mut d_val = 0.0;
        for i in 0..self.waves.len() {
            if self.waves[i].radius > 0.0 {
                if u_val < self.waves[i].radius {
                    u_val = self.waves[i].radius;
                }
            } else {
                if d_val < self.waves[i].radius.abs() {
                    d_val = self.waves[i].radius.abs();
                };
            }
        }
        if u_val == 0.0 || d_val == 0.0 {
            println!("[{}] {}", Red.paint("Danger"), Green.paint("This is a half wave !"));
        }else {
            println!("[AOR] {} convexes, aor = {}", self.waves.len(), Red.paint(format!("{:.2}", u_val /d_val)));
            if u_val / d_val > 3.0 || u_val / d_val < 0.4  {
                println!("[{}] {}",Red.paint("Danger"),  Green.paint("A half wave! (~, 0.4] or [3.0, ~)"));
            }
        }
    }
    */
    pub fn aor_to_result(&self) -> (String, bool) {
        // let mut ret_str = "".to_string();
        let mut u_val = 0.0;
        let mut d_val = 0.0;
        for i in 0..self.waves.len() {
            if self.waves[i].radius > 0.0 {
                if u_val < self.waves[i].radius {
                    u_val = self.waves[i].radius;
                }
            } else {
                let wv = self.waves[i].radius.abs();
                if wv > d_val {
                    d_val = wv;
                };
            }
        }
        let big = u_val.max(d_val);
        let small = u_val.min(d_val);

        if u_val < 0.0001 || d_val < 0.0001 || small / big < 0.25 {
            return (format!("Half, MAX={:.2}", big), false);
        } else {
            let val = big / small;
            return (
                format!(
                    "Full, UpDown={}, Signal={:.2}, MAX={:.2}",
                    self.waves.len(),
                    val,
                    big
                ),
                true,
            );
        }
    }
}
