use std::io::{self, Write};

use crossterm::{
    cursor, queue,
    style::{self, Color, SetBackgroundColor, SetForegroundColor},
    terminal,
};

use minesweeper::board::{Board, GameOutcome};
use minesweeper::cell::CellState;
use minesweeper::settings::{self, CELL_WIDTH, SYMBOL_FLAG, SYMBOL_HIDDEN, SYMBOL_MINE};

fn bg() -> Color {
    let (r, g, b) = settings::BG_COLOR;
    Color::Rgb { r, g, b }
}

fn number_color(n: u8) -> Color {
    if (1..=8).contains(&n) {
        let (r, g, b) = settings::NUMBER_COLORS[n as usize];
        Color::Rgb { r, g, b }
    } else {
        Color::Reset
    }
}

pub fn render(stdout: &mut impl Write, board: &Board) -> io::Result<()> {
    render_with_countdown(stdout, board, None)
}

pub fn render_with_countdown(
    stdout: &mut impl Write,
    board: &Board,
    countdown: Option<u32>,
) -> io::Result<()> {
    let (term_w, _) = terminal::size()?;

    // Calculate offset to center the grid
    let grid_width = board.width * CELL_WIDTH;
    let offset_x = if (term_w as usize) > grid_width {
        ((term_w as usize) - grid_width) / 2
    } else {
        0
    };

    queue!(
        stdout,
        SetBackgroundColor(bg()),
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
    )?;

    // Header
    let header = format!(
        "MINESWEEPER  {}x{}    Mines: {}    Flags: {}/{}",
        board.width, board.height, board.mine_count, board.flags_placed, board.mine_count
    );
    let header_x = if (term_w as usize) > header.len() {
        ((term_w as usize) - header.len()) / 2
    } else {
        0
    };
    queue!(
        stdout,
        cursor::MoveTo(header_x as u16, 0),
        style::SetAttribute(style::Attribute::Bold),
        style::Print(&header),
        style::SetAttribute(style::Attribute::Reset),
        SetBackgroundColor(bg()),
    )?;

    // Grid
    for y in 0..board.height {
        queue!(stdout, cursor::MoveTo(offset_x as u16, (y + 2) as u16))?;

        for x in 0..board.width {
            let cell = board.cell(x, y);
            let is_cursor = x == board.cursor_x && y == board.cursor_y;

            if is_cursor {
                queue!(stdout, SetBackgroundColor(Color::DarkYellow))?;
            }

            match cell.state {
                CellState::Hidden => {
                    queue!(
                        stdout,
                        SetForegroundColor(Color::DarkGrey),
                        style::Print(format!(" {} ", SYMBOL_HIDDEN)),
                    )?;
                }
                CellState::Flagged => {
                    queue!(
                        stdout,
                        SetForegroundColor(Color::Red),
                        style::SetAttribute(style::Attribute::Bold),
                        style::Print(format!(" {} ", SYMBOL_FLAG)),
                    )?;
                }
                CellState::Revealed => {
                    if cell.is_mine {
                        queue!(
                            stdout,
                            SetBackgroundColor(Color::Red),
                            SetForegroundColor(Color::White),
                            style::Print(format!(" {} ", SYMBOL_MINE)),
                        )?;
                    } else if cell.adjacent_mines == 0 {
                        queue!(
                            stdout,
                            SetForegroundColor(Color::Reset),
                            style::Print("   "),
                        )?;
                    } else {
                        queue!(
                            stdout,
                            SetForegroundColor(number_color(cell.adjacent_mines)),
                            style::SetAttribute(style::Attribute::Bold),
                            style::Print(format!(" {} ", cell.adjacent_mines)),
                        )?;
                    }
                }
            }

            queue!(
                stdout,
                style::SetAttribute(style::Attribute::Reset),
                SetBackgroundColor(bg()),
            )?;
        }
    }

    // Footer — controls
    let footer_y = (board.height + 3) as u16;
    let controls = "Arrows/WASD: move | Space: reveal | F: flag | R: restart | Q: quit";
    let controls_x = if (term_w as usize) > controls.len() {
        ((term_w as usize) - controls.len()) / 2
    } else {
        0
    };
    queue!(
        stdout,
        cursor::MoveTo(controls_x as u16, footer_y),
        SetForegroundColor(Color::DarkGrey),
        style::Print(controls),
    )?;

    // Win/loss message
    match board.outcome {
        GameOutcome::Won => {
            let msg = match countdown {
                Some(s) => format!("  YOU WIN! Restarting in {s}...  "),
                None => "  YOU WIN! Press R to play again or Q to quit.  ".to_string(),
            };
            let msg_x = if (term_w as usize) > msg.len() {
                ((term_w as usize) - msg.len()) / 2
            } else {
                0
            };
            queue!(
                stdout,
                cursor::MoveTo(msg_x as u16, footer_y + 2),
                SetBackgroundColor(Color::Green),
                SetForegroundColor(Color::Black),
                style::SetAttribute(style::Attribute::Bold),
                style::Print(&msg),
                style::SetAttribute(style::Attribute::Reset),
                SetBackgroundColor(bg()),
            )?;
        }
        GameOutcome::Lost => {
            let msg = match countdown {
                Some(s) => format!("  GAME OVER! You hit a mine. Restarting in {s}...  "),
                None => "  GAME OVER! You hit a mine. Press R to play again or Q to quit.  "
                    .to_string(),
            };
            let msg_x = if (term_w as usize) > msg.len() {
                ((term_w as usize) - msg.len()) / 2
            } else {
                0
            };
            queue!(
                stdout,
                cursor::MoveTo(msg_x as u16, footer_y + 2),
                SetBackgroundColor(Color::Red),
                SetForegroundColor(Color::White),
                style::SetAttribute(style::Attribute::Bold),
                style::Print(&msg),
                style::SetAttribute(style::Attribute::Reset),
                SetBackgroundColor(bg()),
            )?;
        }
        GameOutcome::Playing => {}
    }

    stdout.flush()?;
    Ok(())
}

pub fn render_menu(stdout: &mut impl Write, selected: usize) -> io::Result<()> {
    let (term_w, term_h) = terminal::size()?;

    queue!(
        stdout,
        SetBackgroundColor(bg()),
        terminal::Clear(terminal::ClearType::All),
    )?;

    let title = "MINESWEEPER";
    let title_x = ((term_w as usize).saturating_sub(title.len())) / 2;
    let start_y = (term_h as usize).saturating_sub(10) / 2;

    queue!(
        stdout,
        cursor::MoveTo(title_x as u16, start_y as u16),
        style::SetAttribute(style::Attribute::Bold),
        style::Print(title),
        style::SetAttribute(style::Attribute::Reset),
        SetBackgroundColor(bg()),
    )?;

    let options = settings::MENU_OPTIONS;

    for (i, option) in options.iter().enumerate() {
        let y = start_y + 2 + i;
        let line = if i == selected {
            format!("> {option}")
        } else {
            format!("  {option}")
        };
        let x = ((term_w as usize).saturating_sub(line.len())) / 2;

        queue!(stdout, cursor::MoveTo(x as u16, y as u16))?;

        if i == selected {
            queue!(
                stdout,
                SetForegroundColor(Color::Yellow),
                style::SetAttribute(style::Attribute::Bold),
                style::Print(&line),
                style::SetAttribute(style::Attribute::Reset),
                SetBackgroundColor(bg()),
            )?;
        } else {
            queue!(stdout, style::Print(&line))?;
        }
    }

    let hint = "Arrow keys to select, Enter to start";
    let hint_x = ((term_w as usize).saturating_sub(hint.len())) / 2;
    queue!(
        stdout,
        cursor::MoveTo(hint_x as u16, (start_y + 6) as u16),
        SetForegroundColor(Color::DarkGrey),
        style::Print(hint),
    )?;

    stdout.flush()?;
    Ok(())
}
