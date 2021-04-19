use crate::util::{StaticSignal};
use crate::send_cmd;

pub struct Signal<S: Iterator> {
    source: S,
    pub points: Vec<S::Item>,
}

impl<S> Signal<S>
where
    S: Iterator,
{
    fn on_tick(&mut self) {
        if let Some(val) = self.source.by_ref().next() {
            self.points.remove(0);
            self.points.push(val);
        }
    }
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub show_chart: bool,
    pub progress: f64,
    pub sparkline: Signal<StaticSignal>,
    pub ticks: u32,
    pub cursor_idx: usize,
    pub cmdbuf: String,
    pub enhanced_graphics: bool,
    pub scroll_up: i32,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool) -> App<'a> {
        let rand_signal = StaticSignal::new();
        let sparkline_points = vec![0u64; 150];
        App {
            title,
            should_quit: false,
            show_chart: true,
            progress: 0.0,
            sparkline: Signal {
                source: rand_signal,
                points: sparkline_points,
            },
            ticks: 0,
            cursor_idx: 0,
            cmdbuf: String::new(),
            enhanced_graphics: enhanced_graphics,
            scroll_up: 0,
        }
    }

    pub fn on_backspace(&mut self) {
        if self.cmdbuf.is_empty() || self.cursor_idx <= 0 {
            return;
        }

        self.cmdbuf.remove(self.cursor_idx-1);
        self.cursor_idx -= 1;
    }
    
    pub fn on_delete(&mut self) {
        if self.cmdbuf.is_empty() || self.cursor_idx >= self.cmdbuf.len() {
            return;
        }
        
        self.cmdbuf.remove(self.cursor_idx);
    }
    
    pub fn on_pageup(&mut self) {
        self.scroll_up += 10;
    }

    pub fn on_pagedown(&mut self) {
        self.scroll_up -= 10;
        if self.scroll_up < 0 {
            self.scroll_up = 0;
        }
    }

    pub fn on_up(&mut self) {
        
    }

    pub fn on_down(&mut self) {
        
    }

    pub fn on_right(&mut self) {
        self.cursor_idx += 1;
        if self.cursor_idx > self.cmdbuf.len() {
            self.cursor_idx = self.cmdbuf.len();
        }
        //self.tabs.next();
    }

    pub fn on_left(&mut self) {
        if self.cursor_idx <= 0 {
            self.cursor_idx = 0;
            return;
        }
        self.cursor_idx -= 1;
        //self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            /*'q' => {
                self.should_quit = true;
            }
            't' => {
                self.show_chart = !self.show_chart;
            }*/
            '\n' => {
                send_cmd(&format!("{}\n", self.cmdbuf));
                self.cmdbuf = format!("");
                self.cursor_idx = 0;
            },
            _ => {
                self.cmdbuf.insert(self.cursor_idx, c);
                self.cursor_idx += 1;
            }
        }
    }

    pub fn on_tick(&mut self) {
        // Update progress
        self.progress += 0.001;
        if self.progress > 1.0 {
            self.progress = 0.0;
        }
        
        self.ticks += 1;

        self.sparkline.on_tick();
    }
}
