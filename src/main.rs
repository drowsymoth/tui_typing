use std::io;
use std::time::{Duration, Instant};

mod all_stats;
mod constants;
mod dict;
mod menu;
mod stats;
mod typing;

use menu::{Menu, MenuCall};
use stats::{Stats, StatsCall};
use typing::{Config, Typ, TypCall};

use core::mem::replace;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

use crate::all_stats::AllStats;

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
    stats: all_stats::AllStats,
    state: State,
    game: Typ,
    menu: Menu,
    config: Config,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.menu = Menu::new();
        self.stats = AllStats::new();

        let tick_rate = Duration::from_millis(1000 / constants::FPS as u64);
        let mut last_tick = Instant::now();
        let mut last_wpm_check = Instant::now();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if matches!(self.state, State::Typing) {
                if self.game.is_end() {
                    self.stats.push(replace(&mut self.game.stats, Stats::new()));
                    self.state = State::GameStats
                }
                if last_wpm_check.elapsed() >= Duration::from_secs(1) {
                    self.game.stats.add_wpm_sample();
                    last_wpm_check = Instant::now();
                }
                if let Config::Time(t, _) = self.config {
                    if self.game.is_time_end(t as u32) {
                        self.game.complete();
                        self.stats.push(replace(&mut self.game.stats, Stats::new()));
                        self.state = State::GameStats;
                    }
                }
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
            State::GameStats => self.stats.render_last(frame),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match self.state {
                State::Typing => match self.game.handle_key_event(key_event) {
                    TypCall::ToMenu => self.state = State::Menu,
                    TypCall::Restart => self.game = Typ::new(&self.config),
                    _ => {}
                },
                State::Menu => match self.menu.handle_key_event(key_event) {
                    MenuCall::Exit => self.exit(),
                    MenuCall::Start(s) => {
                        self.config = s;
                        self.game = Typ::new(&s);
                        self.state = State::Typing;
                    }
                    MenuCall::None => {}
                },
                State::GameStats => match self.game.stats.handle_key_event(key_event) {
                    StatsCall::ToMenu => self.state = State::Menu,
                    StatsCall::Again => {
                        self.game = Typ::new(&self.config);
                        self.state = State::Typing
                    }
                    StatsCall::Exit => self.exit(),
                    StatsCall::None => {}
                },
            },
            _ => {}
        };
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
