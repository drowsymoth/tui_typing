use std::time::{Duration, Instant};
use std::{io, vec};

mod constants;
mod dict;
mod menu;
mod typing;

use menu::{Menu, MenuCall};
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
