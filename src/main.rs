use std::time::{Duration, Instant};
use std::{io, vec};

mod constants;
mod dict;
mod typing;

use typing::{Config, Typ};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
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
    Start(Config),
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
                MenuPage::Words => MenuCall::Start(Config::Words(self.words_count, dict::DICT)),
                MenuPage::Time => MenuCall::Start(Config::Time(self.time, dict::DICT)),
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
                    frame.render_widget(Line::from(text), quantity_area);
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

#[derive(Debug, Default)]
pub struct App {
    state: State,
    game: Typ,
    menu: Menu,
    config: Config,
    exit: bool,
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
            if self.game.is_end() {
                self.state = State::GameStats
            }

            terminal.draw(|frame| self.draw(frame))?;

            if last_wpm_check.elapsed() >= Duration::from_secs(1) {
                match self.state {
                    State::Typing => self.game.wpm_sample(),
                    _ => {}
                }
                last_wpm_check = Instant::now();
            }

            match self.game.kind {
                Config::Time(_, _) => self.game.check_time(),
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
