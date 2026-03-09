use rand::prelude::*;
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
    wpm: Vec<f32>,
    errors: Vec<f32>,
    words_count: u32,
    time: Option<u32>,
    current_word: u32,
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
                time: Some(time),
                current_word: 0,
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
        if !self.user_input[cur_it].is_empty() && self.words_count - 1 != self.current_word {
            if self.user_input[cur_it].chars().count() < self.target[cur_it].chars().count() {
                self.error_incr();
            }
            self.current_word += 1;
            self.user_input.push("".to_string());
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
                self.errors.push(t.elapsed().as_secs_f32());
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
        for i in 0..=self.current_word as usize {
            let max_len = max(
                self.target[i].chars().count(),
                self.user_input[i].chars().count(),
            );
            if counter_words + max_len > limit {
                counter_lines += 1;
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

    fn get_wpm(&self) -> u16 {
        let mut wpm = 0;
        match self.start_time {
            Some(t) => {
                let elapsed = t.elapsed().as_secs_f32();
                wpm = (self.correct as f32 / 5.0 / (elapsed / 60.0)) as u16;
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
        let title = Line::from(" Suffer Fag ".bold());
        let instructions = Line::from(vec![
            " WPM: ".into(),
            self.get_wpm().to_string().into(),
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
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.game = Typ::new(TextType::Words(
            100,
            dict::DICT.iter().map(|s| s.to_string()).collect(),
        ));

        let fps = 10;
        let tick_rate = Duration::from_millis(1000 / fps);
        let mut last_tick = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::ZERO);

            if crossterm::event::poll(timeout)? {
                self.handle_events()?;
            }

            if last_tick.elapsed() >= tick_rate {
                // app.update(); // timer logic
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

    // fn render_game_stats(&self, area: Rect, buf: &mut Buffer) {
    //     let title = Line::from(" Suffer Fag ".bold());
    //     // let instructions = Line::from(vec![
    //     //     " WPM: ".into(),
    //     //     self.get_wpm().to_string().into(),
    //     //     " ".into(),
    //     //     " Accuracy: ".into(),
    //     //     self.get_accur().to_string().into(),
    //     //     " ".into(),
    //     // ]);
    //     let block = Block::bordered()
    //         .title(title.centered())
    //         // .title_bottom(instructions.centered())
    //         .border_set(border::THICK);
    //     let layout_v = Layout::default()
    //         .direction(Direction::Vertical)
    //         .constraints([
    //             Constraint::Fill(1),
    //             Constraint::Length(20),
    //             Constraint::Fill(1),
    //         ])
    //         .split(area);
    //
    //     let layout_h = Layout::default()
    //         .direction(Direction::Horizontal)
    //         .constraints([
    //             Constraint::Fill(1),
    //             Constraint::Length(self.width),
    //             Constraint::Fill(1),
    //         ])
    //         .split(layout_v[1]);
    //     let dataset = vec![
    //         Dataset::default()
    //             .name("wpm")
    //             .marker(symbols::Marker::Braille)
    //             .style(Style::default().fg(Color::Gray))
    //             .data(todo!()),
    //         Dataset::default()
    //             .name("err")
    //             .marker(Marker::Dot)
    //             .style(Style::default().fg(Color::Red))
    //             .data(todo!()),
    //     ];
    // }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.game.render_text(area, buf);
    }
}
