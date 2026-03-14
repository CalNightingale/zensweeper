use std::collections::VecDeque;

use rand::seq::SliceRandom;

use crate::cell::{Cell, CellState};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    Playing,
    Won,
    Lost,
}

pub struct Board {
    pub width: usize,
    pub height: usize,
    pub mine_count: usize,
    pub cells: Vec<Cell>,
    pub outcome: GameOutcome,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub flags_placed: usize,
    pub cells_revealed: usize,
    mines_placed: bool,
}

impl Board {
    pub fn new(width: usize, height: usize, mine_count: usize) -> Self {
        Self {
            width,
            height,
            mine_count,
            cells: vec![Cell::new(); width * height],
            outcome: GameOutcome::Playing,
            cursor_x: width / 2,
            cursor_y: height / 2,
            flags_placed: 0,
            cells_revealed: 0,
            mines_placed: false,
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[self.idx(x, y)]
    }

    fn cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        let i = self.idx(x, y);
        &mut self.cells[i]
    }

    fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::with_capacity(8);
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < self.width && (ny as usize) < self.height
                {
                    result.push((nx as usize, ny as usize));
                }
            }
        }
        result
    }

    fn place_mines(&mut self, safe_x: usize, safe_y: usize) {
        let safe_zone: Vec<(usize, usize)> = {
            let mut zone = vec![(safe_x, safe_y)];
            zone.extend(self.neighbors(safe_x, safe_y));
            zone
        };

        let mut candidates: Vec<usize> = (0..self.width * self.height)
            .filter(|&i| {
                let x = i % self.width;
                let y = i / self.width;
                !safe_zone.contains(&(x, y))
            })
            .collect();

        let mut rng = rand::rng();
        candidates.shuffle(&mut rng);

        for &i in candidates.iter().take(self.mine_count) {
            self.cells[i].is_mine = true;
        }

        self.compute_adjacent_counts();
        self.mines_placed = true;
    }

    fn compute_adjacent_counts(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.cell(x, y).is_mine {
                    continue;
                }
                let count = self
                    .neighbors(x, y)
                    .iter()
                    .filter(|&&(nx, ny)| self.cell(nx, ny).is_mine)
                    .count() as u8;
                self.cell_mut(x, y).adjacent_mines = count;
            }
        }
    }

    pub fn reveal(&mut self, x: usize, y: usize) {
        if self.outcome != GameOutcome::Playing {
            return;
        }

        if !self.mines_placed {
            self.place_mines(x, y);
        }

        let cell = self.cell(x, y);
        if cell.state != CellState::Hidden {
            return;
        }

        if cell.is_mine {
            self.outcome = GameOutcome::Lost;
            // Reveal all mines
            for i in 0..self.cells.len() {
                if self.cells[i].is_mine {
                    self.cells[i].state = CellState::Revealed;
                }
            }
            return;
        }

        // BFS flood-fill
        let mut queue = VecDeque::new();
        queue.push_back((x, y));

        while let Some((cx, cy)) = queue.pop_front() {
            if self.cell(cx, cy).state != CellState::Hidden {
                continue;
            }

            self.cell_mut(cx, cy).state = CellState::Revealed;
            self.cells_revealed += 1;

            if self.cell(cx, cy).adjacent_mines == 0 {
                for (nx, ny) in self.neighbors(cx, cy) {
                    if self.cell(nx, ny).state == CellState::Hidden && !self.cell(nx, ny).is_mine {
                        queue.push_back((nx, ny));
                    }
                }
            }
        }

        if self.cells_revealed == self.width * self.height - self.mine_count {
            self.outcome = GameOutcome::Won;
        }
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        if self.outcome != GameOutcome::Playing {
            return;
        }
        let cell = self.cell_mut(x, y);
        match cell.state {
            CellState::Hidden => {
                cell.state = CellState::Flagged;
                self.flags_placed += 1;
            }
            CellState::Flagged => {
                cell.state = CellState::Hidden;
                self.flags_placed -= 1;
            }
            CellState::Revealed => {}
        }
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        let new_x = self.cursor_x as i32 + dx;
        let new_y = self.cursor_y as i32 + dy;
        if new_x >= 0 && (new_x as usize) < self.width {
            self.cursor_x = new_x as usize;
        }
        if new_y >= 0 && (new_y as usize) < self.height {
            self.cursor_y = new_y as usize;
        }
    }
}
