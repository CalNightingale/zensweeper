use crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub enum Action {
    MoveCursor(Direction),
    Reveal,
    ToggleFlag,
    Quit,
    Restart,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub fn read_action() -> std::io::Result<Action> {
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let action = match key.code {
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                    Some(Action::MoveCursor(Direction::Up))
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                    Some(Action::MoveCursor(Direction::Down))
                }
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => {
                    Some(Action::MoveCursor(Direction::Left))
                }
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
                    Some(Action::MoveCursor(Direction::Right))
                }
                KeyCode::Enter | KeyCode::Char(' ') => Some(Action::Reveal),
                KeyCode::Char('f') => Some(Action::ToggleFlag),
                KeyCode::Char('r') => Some(Action::Restart),
                KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
                _ => None,
            };
            if let Some(a) = action {
                return Ok(a);
            }
        }
    }
}
