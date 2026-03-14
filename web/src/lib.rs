use wasm_bindgen::prelude::*;

use minesweeper::board::{Board, GameOutcome};
use minesweeper::cell::CellState;
use minesweeper::solver;

#[wasm_bindgen]
pub struct Game {
    board: Board,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize, mines: usize) -> Game {
        Game {
            board: Board::new(width, height, mines),
        }
    }

    pub fn width(&self) -> usize {
        self.board.width
    }

    pub fn height(&self) -> usize {
        self.board.height
    }

    pub fn mine_count(&self) -> usize {
        self.board.mine_count
    }

    pub fn flags_placed(&self) -> usize {
        self.board.flags_placed
    }

    /// 0 = playing, 1 = won, 2 = lost
    pub fn outcome(&self) -> u8 {
        match self.board.outcome {
            GameOutcome::Playing => 0,
            GameOutcome::Won => 1,
            GameOutcome::Lost => 2,
        }
    }

    pub fn reveal(&mut self, x: usize, y: usize) {
        self.board.reveal(x, y);
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        self.board.toggle_flag(x, y);
    }

    /// 0 = hidden, 1 = revealed, 2 = flagged
    pub fn cell_state(&self, x: usize, y: usize) -> u8 {
        match self.board.cell(x, y).state {
            CellState::Hidden => 0,
            CellState::Revealed => 1,
            CellState::Flagged => 2,
        }
    }

    pub fn adjacent_mines(&self, x: usize, y: usize) -> u8 {
        self.board.cell(x, y).adjacent_mines
    }

    pub fn is_mine(&self, x: usize, y: usize) -> bool {
        self.board.cell(x, y).is_mine
    }

    pub fn cursor_x(&self) -> usize {
        self.board.cursor_x
    }

    pub fn cursor_y(&self) -> usize {
        self.board.cursor_y
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        self.board.move_cursor(dx, dy);
    }

    pub fn reveal_at_cursor(&mut self) {
        let (x, y) = (self.board.cursor_x, self.board.cursor_y);
        self.board.reveal(x, y);
    }

    pub fn toggle_flag_at_cursor(&mut self) {
        let (x, y) = (self.board.cursor_x, self.board.cursor_y);
        self.board.toggle_flag(x, y);
    }

    /// Returns solver's next move as [type, x, y] where type: 0=reveal, 1=flag.
    /// Returns empty array if no move available.
    pub fn solver_next_move(&self) -> Vec<usize> {
        match solver::next_move(&self.board) {
            Some(solver::Move::Reveal(x, y)) => vec![0, x, y],
            Some(solver::Move::Flag(x, y)) => vec![1, x, y],
            None => vec![],
        }
    }
}
