use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
#[derive(Debug)]
struct Menu {
    tab: usize,
}

fn handle_key_event(key_event: KeyEvent) {
    match key_event.code(){
        KeyCode::Ch
    }
}
