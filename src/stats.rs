use crate::constants;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::{self, border},
    text::{Line, Text},
    widgets::{Axis, Block, Chart, Dataset, GraphType, Paragraph},
    Frame,
};
use std::time::Instant;

#[derive(Debug)]
pub enum StatsCall {
    Again,
    ToMenu,
    Exit,
    None,
}

#[derive(Debug)]
struct ErrorEvent {
    time_stamp: f64,
    char: Option<char>,
    word: String,
}

#[derive(Debug)]
struct Wpm {
    time_stamp: f64,
    value: f64,
}

impl Wpm {
    fn get_vec(input: &Vec<Self>) -> Vec<(f64, f64)> {
        let mut temp: Vec<(f64, f64)> = Vec::new();
        for el in input {
            temp.push((el.time_stamp, el.value));
        }
        temp
    }
}

#[derive(Debug)]
pub struct Stats {
    errors: Vec<ErrorEvent>,
    wpm: Vec<Wpm>,
    start_time: Option<Instant>,
    end_time: Option<u32>,
    correct: u32,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            errors: Vec::new(),
            wpm: Vec::new(),
            start_time: None,
            end_time: None,
            correct: 0,
        }
    }

    pub fn start_time(&mut self) {
        match self.start_time {
            None => self.start_time = Some(Instant::now()),
            _ => {}
        }
    }

    pub fn wpm(&self) -> f32 {
        let mut wpm = 0.0;
        match self.start_time {
            Some(t) => {
                let elapsed = t.elapsed().as_secs_f32();
                wpm = self.correct as f32 / 5.0 / (elapsed / 60.0);
            }
            None => {}
        }
        wpm
    }

    pub fn add_wpm_sample(&mut self) {
        match self.start_time {
            Some(t) => {
                let temp = Wpm {
                    time_stamp: t.elapsed().as_secs_f64(),
                    value: self.wpm() as f64,
                };
                self.wpm.push(temp);
            }
            _ => {}
        }
    }

    fn get_max_wpm(&self) -> f64 {
        let mut max: f64 = 0.0;
        for i in &self.wpm {
            if i.value > max {
                max = i.value;
            }
        }
        max
    }

    pub fn error_incr(&mut self, char: Option<char>, word: String) {
        match self.start_time {
            Some(t) => {
                self.errors.push(ErrorEvent {
                    time_stamp: t.elapsed().as_secs_f64(),
                    char,
                    word: word,
                });
            }
            None => {}
        }
    }

    pub fn get_accur(&self, cur_word: usize) -> u8 {
        let mut accur = 0;
        let error_count = self.errors.len();
        if cur_word > error_count {
            accur =
                ((self.correct - error_count as u32) as f32 * 100.0 / self.correct as f32) as u8;
        }
        accur
    }

    pub fn time(&self) -> u32 {
        match self.start_time {
            Some(t) => t.elapsed().as_secs() as u32,
            None => 0,
        }
    }

    pub fn end_time(&mut self) -> u32 {
        match self.end_time {
            Some(t) => t,
            None => {
                let time = self.time();
                self.end_time = Some(time);
                time
            }
        }
    }

    pub fn is_time_end(&mut self, t: u32) -> bool {
        if t <= self.time() {
            self.end_time();
            true
        } else {
            false
        }
    }

    pub fn set_correct(&mut self, value: u32) {
        self.correct = value;
    }

    pub fn render_game_stats(&self, frame: &mut Frame) {
        let title = Line::from(" Suffer Fag ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);
        let layout_v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(constants::GAME_STATS_HEIGHT),
                Constraint::Fill(1),
            ])
            .split(frame.area());

        let layout_h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(constants::GAME_STATS_WIDTH),
                Constraint::Fill(1),
            ])
            .split(layout_v[1]);
        let [graph, numbers] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(layout_h[1]);

        let graph_height = self.get_max_wpm() + 10.0;
        let mut errors: Vec<(f64, f64)> = Vec::new();
        for i in &self.errors {
            errors.push((i.time_stamp, graph_height / 50.0));
        }
        let temp = Wpm::get_vec(&self.wpm);
        let dataset = vec![
            Dataset::default()
                .name("wpm")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Gray))
                .graph_type(GraphType::Line)
                .data(&temp),
            Dataset::default()
                .name("err")
                .marker(symbols::Marker::Quadrant)
                .style(Style::default().fg(Color::Red))
                .graph_type(GraphType::Bar)
                .data(&errors),
        ];

        let chart = Chart::new(dataset)
            .block(block)
            .x_axis(
                Axis::default()
                    .title("Time")
                    .bounds([1.0, self.wpm.last().unwrap().time_stamp]),
            )
            .y_axis(
                Axis::default()
                    .title("WPM")
                    .bounds([0.0, self.get_max_wpm() + 10.0]),
            );

        frame.render_widget(chart, graph);
        let error_text = Line::from("Errors: ".to_string() + &self.errors.len().to_string());
        let wpm_text = Line::from(
            "WPM: ".to_string() + (self.wpm.last().unwrap().value as u16).to_string().as_str(),
        );
        let time_m = &self.end_time.unwrap() / 60;
        let time_s = &self.end_time.unwrap() % 60;
        let time_text;
        if time_m == 0 {
            time_text =
                Line::from("Time: ".to_string() + time_s.to_string().as_str() + &"s".to_string());
        } else {
            time_text = Line::from(
                "Time: ".to_string()
                    + time_m.to_string().as_str()
                    + "m".to_string().as_str()
                    + " ".to_string().as_str()
                    + time_s.to_string().as_str()
                    + "s".to_string().as_str(),
            );
        }
        let block_numbers = Block::bordered().border_set(border::THICK);
        let par =
            Paragraph::new(Text::from(vec![error_text, time_text, wpm_text])).block(block_numbers);

        // let mut words_err: Vec<Span> = Vec::new();
        // for i in &self.errors {
        //     words_err.push(Span::from(i.word.clone() + " "));
        // }

        // let par = Paragraph::new(Line::from(words_err));

        frame.render_widget(par, numbers);
    }

    pub fn handle_key_event(&self, key_event: KeyEvent) -> StatsCall {
        match key_event.code {
            KeyCode::Esc => StatsCall::ToMenu,
            KeyCode::Enter => StatsCall::Again,
            KeyCode::Char('q') => StatsCall::Exit,
            _ => StatsCall::None,
        }
    }
}
