use rand::prelude::*;
use ratatui::widgets::{Axis, GraphType};
use std::cmp::max;
use std::time::Instant;
use std::vec;

use crate::{constants, dict};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::{self, border},
    text::{Line, Span, Text},
    widgets::{Block, Chart, Dataset, Paragraph, Wrap},
    Frame,
};

#[derive(Debug)]
struct ErrorEvent {
    time_stamp: f64,
    char: Option<char>,
    word: String,
}

#[derive(Debug)]
struct WordArray {
    data: Vec<Vec<char>>,
}

impl WordArray {
    fn is_last_empty(&self) -> bool {
        self.data.last().unwrap().is_empty()
    }

    fn nth_len(&self, n: usize) -> usize {
        match self.data.get(n) {
            Some(v) => v.len(),
            None => 0,
        }
    }

    fn get_char(&self, pos: (usize, usize)) -> Option<char> {
        match self.data.get(pos.0) {
            Some(v) => match v.get(pos.1) {
                Some(v) => Some(*v),
                None => None,
            },
            None => None,
        }
    }

    fn get_word(&self, pos: usize) -> Option<String> {
        match self.data.get(pos) {
            Some(v) => Some((*v).clone().iter().collect()),
            None => None,
        }
    }

    fn default() -> Self {
        Self {
            data: vec![Vec::new()],
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn push_word(&mut self, value: String) {
        self.data.push(value.chars().collect());
    }

    fn pop_word(&mut self) {
        self.data.pop();
    }

    fn push_char(&mut self, value: char) {
        self.data.last_mut().unwrap().push(value);
    }

    fn pop_char(&mut self) {
        self.data.last_mut().unwrap().pop();
    }

    fn from(dict: &'static [&'static str], words_count: usize) -> Self {
        let mut rng = rand::rng();
        let mut temp = WordArray { data: Vec::new() };
        for _ in 0..words_count {
            temp.push_word((**dict.choose(&mut rng).unwrap()).to_string());
        }
        temp
    }

    fn quote(quote: &'static str) -> Self {
        let mut temp = WordArray::default();
        for el in quote.split_whitespace().map(|s| s.to_string()).into_iter() {
            temp.push_word(el);
        }
        temp
    }

    fn next_word(&mut self) {
        self.data.push(Vec::new());
    }

    fn is_first_empty(&self) -> bool {
        match self.data.get(0) {
            Some(v) => v.is_empty(),
            None => true,
        }
    }

    fn clear_nth(&mut self, n: usize) {
        match self.data.get_mut(n) {
            Some(v) => *v = Vec::new(),
            None => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Config {
    Time(usize, &'static [&'static str]),
    Words(usize, &'static [&'static str]),
    Quote(&'static str),
}

impl Default for Config {
    fn default() -> Self {
        Self::Words(100, dict::DICT)
    }
}

#[derive(Debug)]
pub struct Typ {
    user_input: WordArray,
    target: WordArray,
    line_word: Vec<(usize, usize)>,
    start_time: Option<Instant>,
    correct: u32,
    wpm: Vec<(f64, f64)>,
    errors: Vec<ErrorEvent>,
    words_count: usize,
    current_word: usize,
    pub kind: Config,
    end_time: Option<u32>,
    end: bool,
}

impl Typ {
    pub fn new(kind: &Config) -> Self {
        match kind {
            Config::Time(time, dict) => Typ {
                user_input: WordArray::default(),
                target: WordArray::from(dict, time * 500 / 60),
                line_word: Vec::new(),
                start_time: None,
                correct: 0,
                wpm: Vec::new(),
                errors: Vec::new(),
                words_count: time * 500 / 60,
                current_word: 0,
                kind: *kind,
                end_time: None,
                end: false,
            },
            Config::Words(words_count, dict) => Typ {
                user_input: WordArray::default(),
                target: WordArray::from(dict, *words_count),
                line_word: Vec::new(),
                start_time: None,
                correct: 0,
                wpm: Vec::new(),
                errors: Vec::new(),
                words_count: *words_count,
                current_word: 0,
                kind: *kind,
                end_time: None,
                end: false,
            },
            Config::Quote(quote) => {
                let target = WordArray::quote(quote);
                let target_len = target.len();
                Typ {
                    user_input: WordArray::default(),
                    target: target,
                    line_word: Vec::new(),
                    start_time: None,
                    correct: 0,
                    wpm: Vec::new(),
                    errors: Vec::new(),
                    words_count: target_len,
                    current_word: 0,
                    kind: *kind,
                    end_time: None,
                    end: false,
                }
            }
        }
    }

    fn next_word(&mut self) {
        if !self.user_input.is_last_empty() {
            if self.words_count - 1 != self.current_word {
                if self.user_input.nth_len(self.current_word)
                    < self.target.nth_len(self.current_word)
                {
                    self.error_incr(None, self.target.get_word(self.current_word).unwrap());
                }
                self.current_word += 1;
                self.user_input.next_word();
            } else {
                self.complete();
            }
        }
    }

    fn add_char(&mut self, input: char) {
        if self.is_end_line() {
            return;
        }
        self.user_input.push_char(input);

        match self.target.get_char((
            self.current_word,
            self.user_input.nth_len(self.current_word) - 1,
        )) {
            Some(c) => {
                if c != input {
                    self.error_incr(
                        Some(input),
                        self.target.get_word(self.current_word).unwrap(),
                    );
                }
            }
            None => self.error_incr(
                Some(input),
                self.target.get_word(self.current_word).unwrap(),
            ),
        }

        match self.start_time {
            None => self.set_start_time(),
            _ => {}
        }
        self.correct = self.check_correct();
    }

    fn delete_char(&mut self) {
        if self.user_input.is_first_empty() {
            return;
        } else if !self.user_input.is_last_empty() {
            self.user_input.pop_char();
        } else if self.user_input.get_word(self.current_word - 1)
            != self.target.get_word(self.current_word - 1)
        {
            self.user_input.pop_word();
            self.current_word -= 1;
        }
    }

    fn delete_word(&mut self) {
        if self.user_input.is_first_empty() {
            return;
        } else if self.user_input.len() == 1 {
            self.user_input.clear_nth(0);
            return;
        } else if self.user_input.get_word(self.current_word - 1)
            != self.target.get_word(self.current_word - 1)
        {
            self.user_input.pop_word();
            self.current_word -= 1;
        } else if !self.user_input.is_last_empty() {
            self.user_input.clear_nth(self.current_word);
        }
    }

    fn error_incr(&mut self, char: Option<char>, word: String) {
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

    fn check_display(&self) -> Vec<Span> {
        let mut spans: Vec<Span> = Vec::new();
        let (start, end) = self.visible_range();
        for (idx, (ts, us)) in self.target.data[start..end]
            .iter()
            .zip(&self.user_input.data[start..])
            .enumerate()
        {
            for (tc, uc) in ts.iter().zip(us) {
                if tc == uc {
                    spans.push(Span::styled(
                        tc.to_string(),
                        Style::default().fg(Color::Green),
                    ));
                } else {
                    spans.push(Span::styled(
                        tc.to_string(),
                        Style::default().fg(Color::Red),
                    ));
                }
            }
            for tc in ts.iter().skip(us.len()) {
                if idx >= self.current_word - start {
                    spans.push(Span::styled(
                        tc.to_string(),
                        Style::default().fg(Color::Black).bg(Color::DarkGray),
                    ));
                } else {
                    spans.push(Span::styled(
                        tc.to_string(),
                        Style::default().fg(Color::Black).bg(Color::Red),
                    ));
                }
            }
            for uc in us.iter().skip(ts.len()) {
                spans.push(Span::styled(
                    uc.to_string(),
                    Style::default().fg(Color::Red),
                ));
            }
            spans.push(Span::styled(" ".to_string(), Style::default()));
        }
        spans.pop();
        spans.push(Span::styled(
            " ".to_string(),
            Style::default().bg(Color::DarkGray),
        ));

        for ts in self.target.data[start..end]
            .iter()
            .skip(self.user_input.len() - start)
        {
            let temp: String = ts.into_iter().collect();
            spans.push(Span::styled(temp, Style::default().fg(Color::Gray)));
            spans.push(Span::styled(" ".to_string(), Style::default()));
        }
        spans.pop();

        return spans;
    }

    fn check_correct(&self) -> u32 {
        let mut counter: u32 = 0;
        for (ts, us) in self.target.data.iter().zip(&self.user_input.data) {
            for (tc, uc) in ts.iter().zip(us) {
                if tc == uc {
                    counter += 1;
                }
            }
        }
        counter
    }

    fn visible_range(&self) -> (usize, usize) {
        let mut counter_lines = 0;
        let mut counter_words = 0;
        let mut current = self.get_cur_pos();
        if current == 0 {
            current = 1;
        }
        let mut left: Option<usize> = None;
        let mut right: Option<usize> = None;
        let limit: usize = (constants::TYPING_WIDTH - 2) as usize;
        for i in 0..=self.target.len() {
            let max_len = max(self.target.nth_len(i), self.user_input.nth_len(i));
            if counter_words + 1 + max_len > limit {
                counter_lines += 1;
                counter_words = max_len;
            } else {
                counter_words += 1 + max_len;
            }
            if counter_lines == current - 1 && left == None {
                left = Some(i);
            }
            if counter_lines == current + 2 {
                right = Some(i);
                break;
            }
        }
        if right == None {
            right = Some(self.target.len());
        }
        (left.unwrap(), right.unwrap())
    }

    fn get_cur_pos(&self) -> usize {
        let mut counter_lines = 0;
        let limit: usize = (constants::TYPING_WIDTH - 2) as usize;
        let mut counter_words = max(self.target.nth_len(0), self.user_input.nth_len(0));
        for i in 1..=self.current_word as usize {
            let max_len = max(self.target.nth_len(i), self.user_input.nth_len(i));
            if counter_words + 1 + max_len > limit {
                counter_lines += 1;
                counter_words = max_len;
            } else {
                counter_words += 1 + max_len;
            }
        }
        counter_lines
    }

    fn is_end_line(&self) -> bool {
        if self.user_input.nth_len(self.current_word) < self.target.nth_len(self.current_word) {
            return false;
        }
        let limit: usize = (constants::TYPING_WIDTH - 2) as usize;
        let mut counter_words = max(self.target.nth_len(0), self.user_input.nth_len(0));
        for i in 1..=self.current_word as usize {
            let max_len = max(self.target.nth_len(i), self.user_input.nth_len(i));
            if counter_words + 1 + max_len > limit {
                counter_words = max_len;
            } else {
                counter_words += 1 + max_len;
            }
        }
        counter_words >= (constants::TYPING_WIDTH - 2) as usize
    }

    fn set_start_time(&mut self) {
        self.start_time = Some(Instant::now());
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

    fn get_accur(&self) -> u8 {
        let mut accur = 0;
        let error_count = self.errors.len();
        if self.current_word > error_count {
            accur =
                ((self.correct - error_count as u32) as f32 * 100.0 / self.correct as f32) as u8;
        }
        accur
    }

    pub fn render_text(&self, frame: &mut Frame) {
        let mut title = Line::from(" Time: 0s ");
        match self.start_time {
            Some(t) => {
                title = Line::from(vec![
                    " Time: ".into(),
                    t.elapsed().as_secs().to_string().into(),
                    "s ".into(),
                ]);
            }
            None => {}
        }
        let instructions = Line::from(vec![
            " WPM: ".into(),
            (self.wpm() as u16).to_string().into(),
            " ".into(),
            " Accuracy: ".into(),
            self.get_accur().to_string().into(),
            "% ".into(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        let layout_v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(constants::TYPING_HEIGHT),
                Constraint::Fill(1),
            ])
            .split(frame.area());

        let layout_h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(constants::TYPING_WIDTH),
                Constraint::Fill(1),
            ])
            .split(layout_v[1]);

        let mut scroll_v = self.get_cur_pos();
        if scroll_v > 0 {
            scroll_v -= 1;
        }

        let par = Paragraph::new(Text::from(Line::from(self.check_display())))
            .wrap(Wrap { trim: true })
            // .scroll((scroll_v as u16, 0))
            .left_aligned()
            .block(block);

        frame.render_widget(par, layout_h[1]);
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
        let dataset = vec![
            Dataset::default()
                .name("wpm")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Gray))
                .graph_type(GraphType::Line)
                .data(&self.wpm),
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
                    .bounds([1.0, self.wpm.last().unwrap().0]),
            )
            .y_axis(
                Axis::default()
                    .title("WPM")
                    .bounds([0.0, self.get_max_wpm() + 10.0]),
            );

        frame.render_widget(chart, graph);
        let error_text = Line::from("Errors: ".to_string() + &self.errors.len().to_string());
        let wpm_text = Line::from(
            "WPM: ".to_string() + (self.wpm.last().unwrap().1 as u16).to_string().as_str(),
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

    fn get_max_wpm(&self) -> f64 {
        let mut max: f64 = 0.0;
        for i in &self.wpm {
            if i.1 > max {
                max = i.1;
            }
        }
        max
    }

    fn time(&self) -> u32 {
        match self.start_time {
            Some(t) => t.elapsed().as_secs() as u32,
            None => 0,
        }
    }

    fn complete(&mut self) {
        self.end_time();
        self.end = true;
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(' ') => self.next_word(),
            KeyCode::Backspace if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.delete_word()
            }
            KeyCode::Char('h') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.delete_word()
            }
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Char(c) => self.add_char(c),
            _ => {}
        }
    }

    pub fn check_time(&mut self) {
        match self.kind {
            Config::Time(t, _) => {
                if t <= self.time() as usize {
                    self.end_time();
                    self.complete();
                }
            }
            _ => panic!(),
        }
    }

    fn end_time(&mut self) -> u32 {
        match self.end_time {
            Some(t) => t,
            None => {
                let time = self.time();
                self.end_time = Some(time);
                time
            }
        }
    }

    pub fn is_end(&self) -> bool {
        self.end
    }

    pub fn start_time(&self) -> Option<Instant> {
        self.start_time
    }

    pub fn wpm_sample(&mut self) {
        match self.start_time() {
            Some(t) => {
                self.wpm
                    .push((t.elapsed().as_secs_f64(), self.wpm() as f64));
            }
            _ => {}
        }
    }
}

impl Default for Typ {
    fn default() -> Self {
        Typ::new(&Config::default())
    }
}
