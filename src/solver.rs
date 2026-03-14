use rand::seq::SliceRandom;

use crate::board::{Board, GameOutcome};
use crate::cell::CellState;

pub enum Move {
    Reveal(usize, usize),
    Flag(usize, usize),
}

/// Pick the next move for the auto-player.
/// Strategy:
///   1. For each revealed number, if hidden neighbors == number - flagged neighbors, flag them all.
///   2. For each revealed number, if flagged neighbors == number, reveal all hidden neighbors.
///   3. If no logical move found, reveal a random hidden cell.
pub fn next_move(board: &Board) -> Option<Move> {
    if board.outcome != GameOutcome::Playing {
        return None;
    }

    // Pass 1: find cells to flag (all hidden neighbors must be mines)
    for y in 0..board.height {
        for x in 0..board.width {
            let cell = board.cell(x, y);
            if cell.state != CellState::Revealed || cell.adjacent_mines == 0 {
                continue;
            }

            let neighbors = neighbors_of(board, x, y);
            let hidden: Vec<_> = neighbors
                .iter()
                .filter(|&&(nx, ny)| board.cell(nx, ny).state == CellState::Hidden)
                .collect();
            let flagged = neighbors
                .iter()
                .filter(|&&(nx, ny)| board.cell(nx, ny).state == CellState::Flagged)
                .count() as u8;

            let remaining = cell.adjacent_mines - flagged;
            if remaining > 0 && remaining == hidden.len() as u8 {
                let &(fx, fy) = hidden[0];
                return Some(Move::Flag(fx, fy));
            }
        }
    }

    // Pass 2: find safe cells to reveal (flagged neighbors satisfy the number)
    for y in 0..board.height {
        for x in 0..board.width {
            let cell = board.cell(x, y);
            if cell.state != CellState::Revealed || cell.adjacent_mines == 0 {
                continue;
            }

            let neighbors = neighbors_of(board, x, y);
            let flagged = neighbors
                .iter()
                .filter(|&&(nx, ny)| board.cell(nx, ny).state == CellState::Flagged)
                .count() as u8;
            let hidden: Vec<_> = neighbors
                .iter()
                .filter(|&&(nx, ny)| board.cell(nx, ny).state == CellState::Hidden)
                .collect();

            if flagged == cell.adjacent_mines && !hidden.is_empty() {
                let &(rx, ry) = hidden[0];
                return Some(Move::Reveal(rx, ry));
            }
        }
    }

    // Pass 3: no logical deduction — pick a random hidden cell
    let mut candidates: Vec<(usize, usize)> = Vec::new();
    for y in 0..board.height {
        for x in 0..board.width {
            if board.cell(x, y).state == CellState::Hidden {
                candidates.push((x, y));
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    let mut rng = rand::rng();
    candidates.shuffle(&mut rng);
    let (rx, ry) = candidates[0];
    Some(Move::Reveal(rx, ry))
}

fn neighbors_of(board: &Board, x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(8);
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < board.width && (ny as usize) < board.height {
                result.push((nx as usize, ny as usize));
            }
        }
    }
    result
}
