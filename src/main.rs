use rand::prelude::*;
use ratatui::widgets::{Axis, GraphType};
use std::cmp::max;
use std::time::{Duration, Instant};
use std::{default, io, vec};

mod constants;
mod dict;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::{self, border, Marker},
    text::{Line, Span, Text},
    widgets::{Block, Chart, Dataset, Paragraph, Widget, Wrap},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Default, Debug)]
enum State {
    #[default]
    Menu,
    Typing,
    GameStats,
    AllStats,
}

enum TextType {
    Time(u32, Vec<String>),
    Words(u32, Vec<String>),
    Quote(String),
}

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    state: State,
    game: Typ,
}

#[derive(Debug, Default)]
pub struct Typ {
    user_input: Vec<String>,
    target: Vec<String>,
    start_time: Option<Instant>,
    correct: u32,
    wpm: Vec<(f64, f64)>,
    errors: Vec<f64>,
    words_count: u32,
    time: Option<Duration>,
    current_word: u32,
    end: bool,
}

impl Typ {
    pub fn new(kind: TextType) -> Self {
        match kind {
            TextType::Time(time, dict) => Typ {
                user_input: vec!["".to_string()],
                target: Typ::fill_with_shuffle(dict, time * 500 / 60),
                start_time: None,
                correct: 0,
                wpm: Vec::new(),
                errors: Vec::new(),
                words_count: time * 500 / 60,
                time: Some(Duration::from_secs(time as u64)),
                current_word: 0,
                end: false,
            },
            TextType::Words(words_count, dict) => Typ {
                user_input: vec!["".to_string()],
                target: Typ::fill_with_shuffle(dict, words_count),
                start_time: None,
                correct: 0,
                wpm: Vec::new(),
                errors: Vec::new(),
                words_count: words_count,
                time: None,
                current_word: 0,
                end: false,
            },
            TextType::Quote(quote) => {
                let target: Vec<String> = quote.split_whitespace().map(|s| s.to_string()).collect();
                let target_len = target.len() as u32;
                Typ {
                    user_input: vec!["".to_string()],
                    target: target,
                    start_time: None,
                    correct: 0,
                    wpm: Vec::new(),
                    errors: Vec::new(),
                    words_count: target_len,
                    time: None,
                    current_word: 0,
                    end: false,
                }
            }
        }
    }

    fn fill_with_shuffle(dict: Vec<String>, words_count: u32) -> Vec<String> {
        let mut rng = rand::rng();
        let mut target: Vec<String> = Vec::new();
        for _ in 0..words_count {
            target.push((**dict.choose(&mut rng).unwrap()).to_string());
        }
        target
    }

    fn next_word(&mut self) {
        let cur_it = self.current_word as usize;
        if !self.user_input[cur_it].is_empty() {
            if self.words_count - 1 != self.current_word {
                if self.user_input[cur_it].chars().count() < self.target[cur_it].chars().count() {
                    self.error_incr();
                }
                self.current_word += 1;
                self.user_input.push("".to_string());
            } else {
                self.complete();
            }
        }
    }

    fn add_char(&mut self, input: char) {
        let cur_it = self.current_word as usize;
        if self.is_end() {
            return;
        }
        self.user_input[cur_it].push(input);

        match self.target[cur_it]
            .chars()
            .nth(self.user_input[cur_it].chars().count() - 1)
        {
            Some(c) => {
                if c != input {
                    self.error_incr();
                }
            }
            None => self.error_incr(),
        }

        match self.start_time {
            None => self.set_start_time(),
            _ => {}
        }
        self.correct = self.check_correct();
    }

    fn delete_char(&mut self) {
        let cur_it = self.current_word as usize;
        if self.user_input.is_empty() || self.user_input[0].is_empty() {
            return;
        } else if !self.user_input[cur_it].is_empty() {
            self.user_input[cur_it].pop();
        } else if self.user_input[cur_it - 1] != self.target[cur_it - 1] {
            self.user_input.pop();
            self.current_word -= 1;
        }
    }

    fn delete_word(&mut self) {
        let cur_it = self.current_word as usize;
        if self.user_input.is_empty() || self.user_input[0].is_empty() {
            return;
        } else if self.user_input.len() == 1 {
            self.user_input[0] = "".to_string();
            return;
        } else if self.user_input[cur_it - 1] != self.target[cur_it - 1] {
            self.user_input.pop();
            self.current_word -= 1;
        } else if !self.user_input[cur_it].is_empty() {
            self.user_input[cur_it] = "".to_string();
        }
    }

    fn error_incr(&mut self) {
        match self.start_time {
            Some(t) => {
                self.errors.push(t.elapsed().as_secs_f64());
            }
            None => {}
        }
    }

    fn check_display(&self) -> Vec<Span> {
        let mut spans: Vec<Span> = Vec::new();
        for (idx, (ts, us)) in self.target.iter().zip(&self.user_input).enumerate() {
            for (tc, uc) in ts.chars().zip(us.chars()) {
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
            for tc in ts.chars().skip(us.chars().count()) {
                if idx == self.current_word as usize {
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
            for uc in us.chars().skip(ts.chars().count()) {
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

        for ts in self.target.iter().skip(self.user_input.len()) {
            spans.push(Span::styled(ts, Style::default().fg(Color::Gray)));
            spans.push(Span::styled(" ".to_string(), Style::default()));
        }
        spans.pop();

        return spans;
    }

    fn check_correct(&self) -> u32 {
        let mut counter: u32 = 0;
        for (ts, us) in self.target.iter().zip(&self.user_input) {
            for (tc, uc) in ts.chars().zip(us.chars()) {
                if tc == uc {
                    counter += 1;
                }
            }
        }
        counter
    }

    fn get_cur_pos(&self) -> u16 {
        let mut counter_words = 0;
        let mut counter_lines = 0;
        let limit: usize = (constants::TYPING_WIDTH - 2) as usize;
        counter_words = max(
            self.target[0].chars().count(),
            self.user_input[0].chars().count(),
        );
        for i in 1..=self.current_word as usize {
            let max_len = max(
                self.target[i].chars().count(),
                self.user_input[i].chars().count(),
            );
            if counter_words + 1 + max_len > limit {
                counter_lines += 1;
                counter_words = max_len;
            } else {
                counter_words += 1 + max_len;
            }
        }
        counter_lines
    }

    fn is_end(&self) -> bool {
        let cur_it = self.current_word as usize;
        if self.user_input[cur_it].chars().count() < self.target[cur_it].chars().count() {
            return false;
        }
        let mut counter_words = 0;
        let limit: usize = (constants::TYPING_WIDTH - 2) as usize;
        for i in 0..=cur_it {
            let max_len = max(
                self.target[i].chars().count(),
                self.user_input[i].chars().count(),
            );
            if counter_words + max_len > limit {
                counter_words = max_len + 1;
            } else {
                counter_words += max_len;
                match self.target.get(i + 1) {
                    Some(s) => {
                        if counter_words + s.chars().count() < limit {
                            counter_words += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        counter_words >= (constants::TYPING_WIDTH - 2) as usize
    }
    fn set_start_time(&mut self) {
        self.start_time = Some(Instant::now());
    }

    fn get_wpm(&self) -> f32 {
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
        if self.current_word > error_count as u32 {
            accur =
                ((self.correct - error_count as u32) as f32 * 100.0 / self.correct as f32) as u8;
        }
        accur
    }

    fn render_text(&self, area: Rect, buf: &mut Buffer) {
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
            (self.get_wpm() as u16).to_string().into(),
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
            .split(area);

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

        Paragraph::new(Text::from(Line::from(self.check_display())))
            .wrap(Wrap { trim: true })
            .scroll((scroll_v, 0))
            .left_aligned()
            .block(block)
            .render(layout_h[1], buf);
    }

    fn render_game_stats(&self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Suffer Fag ".bold());
        // let instructions = Line::from(vec![
        // " WPM: ".into(),
        // errors.len().to_string().into(),
        // " ".into(),
        // " Accuracy: ".into(),
        // self.get_accur().to_string().into(),
        // " ".into(),
        // ]);
        let block = Block::bordered()
            .title(title.centered())
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);
        let layout_v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(constants::GAME_STATS_HEIGHT),
                Constraint::Fill(1),
            ])
            .split(area);

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
            errors.push((*i, graph_height / 50.0));
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
        Chart::new(dataset)
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
            )
            .render(graph, buf);
        let error_text = Line::from("Errors: ".to_string() + &self.errors.len().to_string());
        let wpm_text = Line::from(
            "WPM: ".to_string() + (self.wpm.last().unwrap().1 as u16).to_string().as_str(),
        );
        let time_m = &self.time.unwrap().as_secs() / 60;
        let time_s = &self.time.unwrap().as_secs() % 60;
        let time_text;
        if time_m == 0 {
            time_text = Line::from(
                "Time: ".to_string() + &self.time.unwrap().as_secs().to_string() + &"s".to_string(),
            );
        } else {
            time_text = Line::from(
                "Time: ".to_string()
                    + &time_m.to_string()
                    + &"m".to_string()
                    + &" ".to_string()
                    + &time_s.to_string()
                    + &"s".to_string(),
            );
        }
        let block_numbers = Block::bordered()
            // .title(title.centered())
            .border_set(border::THICK);
        Paragraph::new(Text::from(vec![error_text, time_text, wpm_text]))
            .block(block_numbers)
            .render(numbers, buf);
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

    fn complete(&mut self) {
        self.end = true;
        if self.time == None {
            self.time = Some(self.start_time.unwrap().elapsed());
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.game = Typ::new(TextType::Words(
            100,
            dict::DICT.iter().map(|s| s.to_string()).collect(),
        ));
        // self.game = Typ::new(TextType::Quote(dict::QUOTE.to_string()));

        self.state = State::Typing;

        let tick_rate = Duration::from_millis(1000 / constants::FPS as u64);
        let mut last_tick = Instant::now();
        let mut last_wpm_check = Instant::now();
        while !self.exit {
            if self.game.end {
                self.state = State::GameStats
            }

            terminal.draw(|frame| self.draw(frame))?;

            if last_wpm_check.elapsed() >= Duration::from_secs(1) {
                match self.state {
                    State::Typing => match self.game.start_time {
                        Some(t) => {
                            self.game
                                .wpm
                                .push((t.elapsed().as_secs_f64(), self.game.get_wpm() as f64));
                        }
                        _ => {}
                    },
                    _ => {}
                }
                last_wpm_check = Instant::now();
            }

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::ZERO);

            if crossterm::event::poll(timeout)? {
                self.handle_events()?;
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(' ') => self.game.next_word(),
            KeyCode::Char('q') => self.exit(),
            KeyCode::Backspace if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.game.delete_word()
            }
            KeyCode::Char('h') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.game.delete_word()
            }
            KeyCode::Backspace => self.game.delete_char(),
            KeyCode::Char(c) => self.game.add_char(c),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.state {
            State::Typing => self.game.render_text(area, buf),
            State::GameStats => self.game.render_game_stats(area, buf),
            _ => panic!(),
        }
    }
}
