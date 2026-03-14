#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub is_mine: bool,
    pub state: CellState,
    pub adjacent_mines: u8,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            is_mine: false,
            state: CellState::Hidden,
            adjacent_mines: 0,
        }
    }
}
