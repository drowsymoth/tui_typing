// ANCHOR: imports
// use rand::Rng;
use rand::prelude::*;
use std::{io, vec};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget, Wrap},
};
// ANCHOR_END: imports

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

// ANCHOR: app
#[derive(Debug, Default)]
pub struct App {
    user_input: Vec<String>,
    errors: usize,
    target: Vec<String>,
    words_count: usize,
    current_word: usize,
    exit: bool,
}
// ANCHOR_END: app

// ANCHOR: impl App
impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.words_count = 1000;
        self.current_word = 0;

        let dict = vec![
            "the", "be", "of", "and", "a", "to", "in", "he", "have", "it", "that", "for", "they",
            "I", "with", "as", "not", "on", "she", "at", "by", "this", "we", "you", "do", "but",
            "from", "or", "which", "one", "would", "all", "will", "there", "say", "who", "make",
            "when", "can", "more", "if", "no", "man", "out", "other", "so", "what", "time", "up",
            "go", "about", "than", "into", "could", "state", "only", "new", "year", "some", "take",
            "come", "these", "know", "see", "use", "get", "like", "then", "first", "any", "work",
            "now", "may", "such", "give", "over", "think", "most", "even", "find", "day", "also",
            "after", "way", "many", "must", "look", "before", "great", "back", "through", "long",
            "where", "much", "should", "well", "people", "down", "own", "just", "because", "good",
            "each", "those", "feel", "seem", "how", "high", "too", "place", "little", "world",
            "very", "still", "nation", "hand", "old", "life", "tell", "write", "become", "here",
            "show", "house", "both", "between", "need", "mean", "call", "develop", "under", "last",
            "right", "move", "thing", "general", "school", "never", "same", "another", "begin",
            "while", "number", "part", "turn", "real", "leave", "might", "want", "point", "form",
            "off", "child", "few", "small", "since", "against", "ask", "late", "home", "interest",
            "large", "person", "end", "open", "public", "follow", "during", "present", "without",
            "again", "hold", "govern", "around", "possible", "head", "consider", "word", "program",
            "problem", "however", "lead", "system", "set", "order", "eye", "plan", "run", "keep",
            "face", "fact", "group", "play", "stand", "increase", "early", "course", "change",
            "help", "line",
        ];

        let mut rng = rand::rng();

        for _ in 0..self.words_count {
            self.target
                .push((**dict.choose(&mut rng).unwrap()).to_string());
        }

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    // ANCHOR: handle_key_event fn
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(' ') => self.next_word(),
            KeyCode::Char('q') => self.exit(),
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
    // ANCHOR_END: handle_key_event fn
    fn next_word(&mut self) {
        if !self.user_input[self.current_word].is_empty() {
            if self.user_input[self.current_word].chars().count()
                < self.target[self.current_word].chars().count()
            {
                self.error_incr();
            }
            self.current_word += 1;
            self.user_input.push("".to_string());
        }
    }

    fn add_char(&mut self, input: char) {
        if self.user_input.len() > self.current_word {
            self.user_input[self.current_word].push(input);
        } else {
            self.user_input.push(input.to_string());
        }

        match self.target[self.current_word]
            .chars()
            .nth(self.user_input[self.current_word].chars().count() - 1)
        {
            Some(c) => {
                if c != input {
                    self.error_incr();
                }
            }
            None => self.error_incr(),
        }
    }

    fn delete_char(&mut self) {
        if self.user_input.is_empty() || self.user_input[0].is_empty() {
            return;
        } else if !self.user_input[self.current_word].is_empty() {
            self.user_input[self.current_word].pop();
        } else if self.user_input[self.current_word - 1] != self.target[self.current_word - 1] {
            self.user_input.pop();
            self.current_word -= 1;
        }
    }

    fn delete_word(&mut self) {
        if self.user_input.is_empty() || self.user_input[0].is_empty() {
            return;
        } else if self.user_input.len() == 1 {
            self.user_input[0] = "".to_string();
            return;
        } else if self.user_input[self.current_word - 1] != self.target[self.current_word - 1] {
            self.user_input.pop();
            self.current_word -= 1;
        } else if !self.user_input[self.current_word].is_empty() {
            self.user_input[self.current_word] = "".to_string();
        }
    }

    fn error_incr(&mut self) {
        self.errors += 1;
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn compare(&mut self) {}
}
// ANCHOR_END: impl App

// ANCHOR: impl Widget
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Suffer Fag ".bold());
        let instructions = Line::from(vec![
            " Errors: ".into(),
            self.errors.to_string().into(),
            " ".into(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let layout_v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(10),
                Constraint::Fill(1),
            ])
            .split(area);

        let layout_h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(100),
                Constraint::Fill(1),
            ])
            .split(layout_v[1]);

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
            for tc in ts.chars().skip(us.len()) {
                if idx == self.current_word {
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
            for uc in us.chars().skip(ts.len()) {
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
        Paragraph::new(Text::from(Line::from(spans)))
            .wrap(Wrap { trim: true })
            .left_aligned()
            .block(block)
            .render(layout_h[1], buf);
    }
}
// ANCHOR_END: impl Widget
