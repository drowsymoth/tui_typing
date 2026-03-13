use rand::prelude::*;
use ratatui::widgets::{Axis, GraphType};
use std::cmp::max;
use std::time::{self, Duration, Instant};
use std::{io, vec};

mod constants;
mod dict;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    symbols::{self, border},
    text::{Line, Span, Text},
    widgets::{Block, Chart, Dataset, Paragraph, Tabs, Wrap},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Default, Debug, Clone, Copy)]
enum MenuPage {
    #[default]
    Words,
    Time,
    Quote,
}

impl MenuPage {
    const ALL: [Self; 3] = [Self::Words, Self::Time, Self::Quote];

    fn next(&self) -> Self {
        let count = Self::ALL.len();
        let i = ((*self) as usize + 1) % count;
        Self::ALL[i]
    }

    fn prev(&self) -> Self {
        let count = Self::ALL.len();
        let i = (count + (*self) as usize - 1) % count;
        Self::ALL[i]
    }

    fn title(&self) -> &'static str {
        match self {
            MenuPage::Words => "Words",
            MenuPage::Time => "Time",
            MenuPage::Quote => "Quotes",
        }
    }
}

enum MenuCall {
    Exit,
    Start(TextType),
    None,
}

#[derive(Debug, Default)]
enum Selected {
    #[default]
    Tabs,
    Quantity,
}

impl Selected {
    fn next(&self) -> Option<Self> {
        match self {
            Selected::Tabs => Some(Selected::Quantity),
            Selected::Quantity => None,
        }
    }

    fn prev(&self) -> Option<Self> {
        match self {
            Selected::Tabs => None,
            Selected::Quantity => Some(Selected::Tabs),
        }
    }
}

#[derive(Debug, Default)]
struct Menu {
    page: MenuPage,
    words_count: usize,
    time: usize,
    selected: Selected,
}

impl Menu {
    fn new() -> Self {
        Self {
            page: MenuPage::Words,
            words_count: 100,
            time: 30,
            selected: Selected::Tabs,
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> MenuCall {
        match key_event.code {
            KeyCode::Enter => match self.page {
                MenuPage::Words => MenuCall::Start(TextType::Words(self.words_count, dict::DICT)),
                MenuPage::Time => MenuCall::Start(TextType::Time(self.time, dict::DICT)),
                MenuPage::Quote => {
                    //TODO
                    MenuCall::None
                }
            },
            KeyCode::Char('j') => {
                match self.selected.next() {
                    Some(s) => self.selected = s,
                    None => {}
                }
                MenuCall::None
            }
            KeyCode::Char('k') => {
                match self.selected.prev() {
                    Some(s) => self.selected = s,
                    None => {}
                }
                MenuCall::None
            }
            KeyCode::Char('q') => MenuCall::Exit,
            KeyCode::Char('h') => {
                self.handle_left();
                MenuCall::None
            }
            KeyCode::Char('l') => {
                self.handle_right();
                MenuCall::None
            }
            _ => MenuCall::None,
        }
    }

    fn handle_left(&mut self) {
        match self.selected {
            Selected::Tabs => {
                self.page = self.page.prev();
            }
            Selected::Quantity => match self.page {
                MenuPage::Words => {
                    if self.words_count > 10 {
                        self.words_count -= 10;
                    }
                }
                MenuPage::Time => {
                    if self.time > 10 {
                        self.time -= 10;
                    }
                }
                MenuPage::Quote => {
                    //TODO
                }
            },
        }
    }

    fn handle_right(&mut self) {
        match self.selected {
            Selected::Tabs => {
                self.page = self.page.next();
            }
            Selected::Quantity => match self.page {
                MenuPage::Words => {
                    self.words_count += 10;
                }
                MenuPage::Time => {
                    self.time += 10;
                }
                MenuPage::Quote => {
                    //TODO
                }
            },
        }
    }

    fn render_menu(&self, frame: &mut Frame) {
        // let block = Block::bordered().border_set(border::DOUBLE);
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

        let [tabs_area, quantity_area, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(layout_h[1]);

        let tab_cur = Span::styled("<".to_string() + self.page.title() + ">", {
            match self.selected {
                Selected::Tabs => Modifier::REVERSED,
                Selected::Quantity => Modifier::HIDDEN,
            }
        });
        let tab_cur = Line::from(Span::styled("type: ", Style::new().fg(Color::White)) + tab_cur);
        let type_len = "type: ".len();

        frame.render_widget(tab_cur, tabs_area);

        match self.selected {
            Selected::Quantity => match self.page {
                MenuPage::Words => {
                    let text = "<".to_string() + self.words_count.to_string().as_str() + ">";
                    let shift = Span::styled(" ".repeat(type_len), Style::new());
                    let quantity = Span::styled(text, {
                        match self.selected {
                            Selected::Tabs => Modifier::HIDDEN,
                            Selected::Quantity => Modifier::REVERSED,
                        }
                    });
                    let line = Line::from(vec![shift, quantity]);
                    frame.render_widget(line, quantity_area);
                }
                MenuPage::Time => {
                    let text = "<".to_string() + self.time.to_string().as_str() + "s" + ">";
                    let shift = Span::styled(" ".repeat(type_len), Style::new());
                    let quantity = Span::styled(text, {
                        match self.selected {
                            Selected::Tabs => Modifier::HIDDEN,
                            Selected::Quantity => Modifier::REVERSED,
                        }
                    });
                    let line = Line::from(vec![shift, quantity]);
                    frame.render_widget(line, quantity_area);
                }
                MenuPage::Quote => {
                    let text = "todo";
                    let quantity_tab = Tabs::new(vec![text])
                        .style(Color::White)
                        .highlight_style(Modifier::REVERSED)
                        .select(0);
                    frame.render_widget(quantity_tab, quantity_area);
                }
            },
            _ => {}
        }
    }
}

#[derive(Default, Debug)]
enum State {
    #[default]
    Menu,
    Typing,
    GameStats,
    // AllStats,
}

#[derive(Debug, Clone, Copy)]
enum TextType {
    Time(usize, &'static [&'static str]),
    Words(usize, &'static [&'static str]),
    Quote(&'static str),
}

impl Default for TextType {
    fn default() -> Self {
        Self::Words(100, dict::DICT)
    }
}

#[derive(Debug, Default)]
pub struct App {
    state: State,
    game: Typ,
    menu: Menu,
    config: TextType,
    exit: bool,
}

#[derive(Debug)]
pub struct Typ {
    user_input: Vec<String>,
    target: Vec<String>,
    start_time: Option<Instant>,
    correct: u32,
    wpm: Vec<(f64, f64)>,
    errors: Vec<f64>,
    words_count: usize,
    current_word: usize,
    kind: TextType,
    end_time: Option<u32>,
    end: bool,
}

impl Typ {
    fn new(kind: &TextType) -> Self {
        match kind {
            TextType::Time(time, dict) => Typ {
                user_input: vec!["".to_string()],
                target: Typ::fill_with_shuffle(dict, time * 500 / 60),
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
            TextType::Words(words_count, dict) => Typ {
                user_input: vec!["".to_string()],
                target: Typ::fill_with_shuffle(dict, *words_count),
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
            TextType::Quote(quote) => {
                let target: Vec<String> = quote.split_whitespace().map(|s| s.to_string()).collect();
                let target_len = target.len();
                Typ {
                    user_input: vec!["".to_string()],
                    target: target,
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

    fn fill_with_shuffle(dict: &'static [&'static str], words_count: usize) -> Vec<String> {
        let mut rng = rand::rng();
        let mut target: Vec<String> = Vec::new();
        for _ in 0..words_count {
            target.push((**dict.choose(&mut rng).unwrap()).to_string());
        }
        target
    }

    fn next_word(&mut self) {
        if !self.user_input[self.current_word].is_empty() {
            if self.words_count - 1 != self.current_word {
                if self.user_input[self.current_word].chars().count()
                    < self.target[self.current_word].chars().count()
                {
                    self.error_incr();
                }
                self.current_word += 1;
                self.user_input.push("".to_string());
            } else {
                self.end_time();
                self.complete();
            }
        }
    }

    fn add_char(&mut self, input: char) {
        if self.is_end() {
            return;
        }
        self.user_input[self.current_word].push(input);

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

        match self.start_time {
            None => self.set_start_time(),
            _ => {}
        }
        self.correct = self.check_correct();
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
        let mut counter_lines = 0;
        let limit: usize = (constants::TYPING_WIDTH - 2) as usize;
        let mut counter_words = max(
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
        if self.user_input[self.current_word as usize].chars().count()
            < self.target[self.current_word as usize].chars().count()
        {
            return false;
        }
        let limit: usize = (constants::TYPING_WIDTH - 2) as usize;
        let mut counter_words = max(
            self.target[0].chars().count(),
            self.user_input[0].chars().count(),
        );
        for i in 1..=self.current_word as usize {
            let max_len = max(
                self.target[i].chars().count(),
                self.user_input[i].chars().count(),
            );
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
        if self.current_word > error_count {
            accur =
                ((self.correct - error_count as u32) as f32 * 100.0 / self.correct as f32) as u8;
        }
        accur
    }

    fn render_text(&self, frame: &mut Frame) {
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
            .scroll((scroll_v, 0))
            .left_aligned()
            .block(block);

        frame.render_widget(par, layout_h[1]);
    }

    fn render_game_stats(&self, frame: &mut Frame) {
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
        let time_m = &self.time() / 60;
        let time_s = &self.time() % 60;
        let time_text;
        if time_m == 0 {
            time_text = Line::from(
                "Time: ".to_string() + &self.end_time.unwrap().to_string() + &"s".to_string(),
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
        let block_numbers = Block::bordered().border_set(border::THICK);
        let par =
            Paragraph::new(Text::from(vec![error_text, time_text, wpm_text])).block(block_numbers);

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
        self.end = true;
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
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

    fn check_time(&mut self) {
        match self.kind {
            TextType::Time(t, _) => {
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
}

impl Default for Typ {
    fn default() -> Self {
        Typ::new(&TextType::default())
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // self.game = Typ::new(&self.config);
        self.menu = Menu::new();
        // self.state = State::Typing;

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

            match self.game.kind {
                TextType::Time(_, _) => self.game.check_time(),
                _ => {}
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
        match self.state {
            State::Menu => self.menu.render_menu(frame),
            State::Typing => self.game.render_text(frame),
            State::GameStats => self.game.render_game_stats(frame),
            // _ => panic!(),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match self.state {
                    State::Typing => {
                        self.game.handle_key_event(key_event);
                    }
                    State::Menu => match self.menu.handle_key_event(key_event) {
                        MenuCall::Exit => self.exit(),
                        MenuCall::Start(s) => {
                            self.game = Typ::new(&s);
                            self.state = State::Typing;
                        }
                        _ => {}
                    },
                    State::GameStats => match key_event.code {
                        KeyCode::Char('q') => self.exit(),
                        _ => {}
                    },
                    _ => {}
                }
                // self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
