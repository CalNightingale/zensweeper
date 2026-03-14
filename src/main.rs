mod input;
mod render;

use minesweeper::board;
use minesweeper::solver;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    cursor, event, execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use board::{Board, GameOutcome};
use input::{Action, Direction};

/// Zen mode speed: inputs per second (arrow keys, reveals, flags each count as one input).
const ZEN_INPUTS_PER_SEC: f64 = 10.0;

/// Countdown seconds after game ends in zen mode before restarting.
const ZEN_END_COUNTDOWN: u32 = 3;

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

fn run_interactive() -> io::Result<()> {
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

fn run_zen() -> io::Result<()> {
    let _guard = TerminalGuard::new()?;
    let mut stdout = io::stdout();
    let input_delay = Duration::from_secs_f64(1.0 / ZEN_INPUTS_PER_SEC);

    // Expert mode: 30x16, 99 mines
    loop {
        let mut board = Board::new(30, 16, 99);

        loop {
            render::render(&mut stdout, &board)?;

            if board.outcome != GameOutcome::Playing {
                for s in (1..=ZEN_END_COUNTDOWN).rev() {
                    render::render_with_countdown(&mut stdout, &board, Some(s))?;
                    if poll_quit(Duration::from_secs(1))? {
                        return Ok(());
                    }
                }
                break;
            }

            let Some(mv) = solver::next_move(&board) else {
                break;
            };

            let (tx, ty) = match mv {
                solver::Move::Reveal(x, y) | solver::Move::Flag(x, y) => (x, y),
            };

            // Walk cursor to target one step at a time
            while board.cursor_x != tx || board.cursor_y != ty {
                if board.cursor_x < tx {
                    board.move_cursor(1, 0);
                } else if board.cursor_x > tx {
                    board.move_cursor(-1, 0);
                } else if board.cursor_y < ty {
                    board.move_cursor(0, 1);
                } else {
                    board.move_cursor(0, -1);
                }
                render::render(&mut stdout, &board)?;
                if poll_quit(input_delay)? {
                    return Ok(());
                }
            }

            // Perform the action (also one input tick)
            if poll_quit(input_delay)? {
                return Ok(());
            }
            match mv {
                solver::Move::Reveal(x, y) => board.reveal(x, y),
                solver::Move::Flag(x, y) => board.toggle_flag(x, y),
            }
        }
    }
}

/// Wait for `duration`, but return true immediately if the user presses Q/Esc.
/// Continuously drains all events during the wait so held keys can't speed things up.
fn poll_quit(duration: Duration) -> io::Result<bool> {
    let deadline = Instant::now() + duration;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(false);
        }
        if event::poll(remaining)? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press
                    && matches!(key.code, event::KeyCode::Char('q') | event::KeyCode::Esc)
                {
                    return Ok(true);
                }
            }
        }
    }
}

fn run_bench(n: usize) {
    let mut wins = 0usize;
    let mut total_revealed_on_loss = 0usize;
    let mut losses = 0usize;
    let total_safe = 30 * 16 - 99; // Expert: 381 safe cells

    for _ in 0..n {
        let mut board = Board::new(30, 16, 99);
        loop {
            let Some(mv) = solver::next_move(&board) else {
                break;
            };
            match mv {
                solver::Move::Reveal(x, y) => board.reveal(x, y),
                solver::Move::Flag(x, y) => board.toggle_flag(x, y),
            }
            if board.outcome != GameOutcome::Playing {
                break;
            }
        }
        match board.outcome {
            GameOutcome::Won => wins += 1,
            GameOutcome::Lost => {
                losses += 1;
                total_revealed_on_loss += board.cells_revealed;
            }
            GameOutcome::Playing => {
                // Solver gave up (shouldn't happen with smart_guess)
                losses += 1;
                total_revealed_on_loss += board.cells_revealed;
            }
        }
    }

    let win_rate = wins as f64 / n as f64 * 100.0;
    let avg_revealed = if losses > 0 {
        total_revealed_on_loss as f64 / losses as f64
    } else {
        0.0
    };
    println!("Expert: {n} games, {wins} wins, {win_rate:.1}% win rate");
    println!("Average revealed on loss: {avg_revealed:.0}/{total_safe}");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Some(pos) = args.iter().position(|a| a == "--bench") {
        let n = args
            .get(pos + 1)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(100);
        run_bench(n);
        return;
    }

    let zen = args.iter().any(|arg| arg == "--zen");

    let result = if zen { run_zen() } else { run_interactive() };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
