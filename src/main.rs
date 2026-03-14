mod board;
mod cell;
mod input;
mod render;

use std::io;

use crossterm::{
    cursor, execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use board::Board;
use input::{Action, Direction};

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

fn select_difficulty(stdout: &mut impl io::Write) -> io::Result<Option<(usize, usize, usize)>> {
    let presets = [(9, 9, 10), (16, 16, 40), (30, 16, 99)];
    let mut selected: usize = 0;

    loop {
        render::render_menu(stdout, selected)?;

        match input::read_action()? {
            Action::MoveCursor(Direction::Up) => {
                selected = selected.saturating_sub(1);
            }
            Action::MoveCursor(Direction::Down) => {
                if selected < 2 {
                    selected += 1;
                }
            }
            Action::Reveal => return Ok(Some(presets[selected])),
            Action::Quit => return Ok(None),
            _ => {}
        }
    }
}

fn run() -> io::Result<()> {
    let _guard = TerminalGuard::new()?;
    let mut stdout = io::stdout();

    loop {
        let Some((width, height, mines)) = select_difficulty(&mut stdout)? else {
            return Ok(());
        };

        let mut board = Board::new(width, height, mines);

        loop {
            render::render(&mut stdout, &board)?;

            match input::read_action()? {
                Action::MoveCursor(dir) => {
                    let (dx, dy) = match dir {
                        Direction::Up => (0, -1),
                        Direction::Down => (0, 1),
                        Direction::Left => (-1, 0),
                        Direction::Right => (1, 0),
                    };
                    board.move_cursor(dx, dy);
                }
                Action::Reveal => {
                    board.reveal(board.cursor_x, board.cursor_y);
                }
                Action::ToggleFlag => {
                    board.toggle_flag(board.cursor_x, board.cursor_y);
                }
                Action::Restart => break,
                Action::Quit => return Ok(()),
            }
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
