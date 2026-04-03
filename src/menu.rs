use std::{fmt::Formatter, vec};

use crate::{
    Config, constants,
    dict::{self, ARTICLES},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

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

pub enum MenuCall {
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
pub struct Menu {
    page: MenuPage,
    words_count: usize,
    time: usize,
    article: usize,
    selected: Selected,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            page: MenuPage::Words,
            words_count: 100,
            time: 30,
            article: 0,
            selected: Selected::Tabs,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> MenuCall {
        match key_event.code {
            KeyCode::Enter => match self.page {
                MenuPage::Words => MenuCall::Start(Config::Words(self.words_count, dict::DICT)),
                MenuPage::Time => MenuCall::Start(Config::Time(self.time, dict::DICT)),
                MenuPage::Quote => MenuCall::Start(Config::Quote(dict::ARTICLES[self.article])),
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
                    if self.article > 0 {
                        self.article -= 1;
                    }
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
                    if self.article < ARTICLES.len() - 1 {
                        self.article += 1;
                    }
                }
            },
        }
    }

    fn menu_area(area: Rect) -> Rect {
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
        layout_h[1]
    }

    pub fn render_menu(&self, frame: &mut Frame) {
        // let block = Block::bordered().border_set(border::DOUBLE);

        let [tabs_area, quantity_area, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(Self::menu_area(frame.area()));

        let tab_words = Span::styled("<".to_string() + MenuPage::Words.title() + ">", {
            match self.selected {
                Selected::Tabs => match self.page {
                    MenuPage::Words => Modifier::REVERSED,
                    _ => Modifier::HIDDEN,
                },
                Selected::Quantity => Modifier::HIDDEN,
            }
        });
        let tab_time = Span::styled("<".to_string() + MenuPage::Time.title() + ">", {
            match self.selected {
                Selected::Tabs => match self.page {
                    MenuPage::Time => Modifier::REVERSED,
                    _ => Modifier::HIDDEN,
                },
                Selected::Quantity => Modifier::HIDDEN,
            }
        });
        let tab_quotes = Span::styled("<".to_string() + MenuPage::Quote.title() + ">", {
            match self.selected {
                Selected::Tabs => match self.page {
                    MenuPage::Quote => Modifier::REVERSED,
                    _ => Modifier::HIDDEN,
                },
                Selected::Quantity => Modifier::HIDDEN,
            }
        });
        let tab_cur = Line::from(
            Span::styled("type: ", Style::new().fg(Color::White))
                + tab_words
                + Span::from(" ")
                + tab_time
                + Span::from(" ")
                + tab_quotes,
        );
        let type_len = "type: ".len();

        frame.render_widget(tab_cur, tabs_area);

        match self.page {
            MenuPage::Words => {
                let text = format!("<{}>", self.words_count);
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
                let text = format!("<{}>", self.time);
                let shift = Span::styled(
                    " ".repeat(type_len + MenuPage::Time.title().len() + 4),
                    Style::new(),
                );
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
                let text = format!("<Article {}>", self.article);
                let shift = Span::styled(
                    " ".repeat(
                        type_len + MenuPage::Time.title().len() + MenuPage::Words.title().len() + 6,
                    ),
                    Style::new(),
                );
                let quantity = Span::styled(text, {
                    match self.selected {
                        Selected::Tabs => Modifier::HIDDEN,
                        Selected::Quantity => Modifier::REVERSED,
                    }
                });
                let line = Line::from(vec![shift, quantity]);
                frame.render_widget(line, quantity_area);
            }
        }
    }
}
