use crate::stats::Stats;
use ratatui::Frame;
// use serde::{Deserialize, Serialize};
#[derive(Default, Debug)]
pub struct AllStats {
    data: Vec<Stats>,
}

impl AllStats {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(20),
        }
    }

    pub fn push(&mut self, value: Stats) {
        self.data.push(value);
    }

    pub fn render_last(&self, frame: &mut Frame) {
        self.data.last().unwrap().render_game_stats(frame);
    }

    // pub fn write_file() {}
}
