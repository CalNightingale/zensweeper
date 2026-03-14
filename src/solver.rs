use crate::board::{Board, GameOutcome};
use crate::cell::CellState;

pub enum Move {
    Reveal(usize, usize),
    Flag(usize, usize),
}

/// A constraint: "exactly `mines` of `cells` are mines."
struct Constraint {
    mines: u8,
    cells: Vec<(usize, usize)>,
}

/// Pick the next move for the auto-player.
/// Strategy:
///   1. Single-cell deductions (trivial flag/reveal).
///   2. Constraint subset reasoning (overlap between pairs of numbered cells).
///   3. Random guess as last resort.
pub fn next_move(board: &Board) -> Option<Move> {
    if board.outcome != GameOutcome::Playing {
        return None;
    }

    // Try single-cell deductions first (cheap).
    if let Some(mv) = single_cell_deduction(board) {
        return Some(mv);
    }

    // Build constraints and try subset reasoning.
    if let Some(mv) = constraint_subset_deduction(board) {
        return Some(mv);
    }

    // No logical move — make the safest guess.
    smart_guess(board)
}

/// For each revealed number, check if we can trivially flag or reveal neighbors.
fn single_cell_deduction(board: &Board) -> Option<Move> {
    for y in 0..board.height {
        for x in 0..board.width {
            let cell = board.cell(x, y);
            if cell.state != CellState::Revealed || cell.adjacent_mines == 0 {
                continue;
            }

            let neighbors = neighbors_of(board, x, y);
            let hidden: Vec<_> = neighbors
                .iter()
                .copied()
                .filter(|&(nx, ny)| board.cell(nx, ny).state == CellState::Hidden)
                .collect();
            let flagged = neighbors
                .iter()
                .filter(|&&(nx, ny)| board.cell(nx, ny).state == CellState::Flagged)
                .count() as u8;

            let remaining = cell.adjacent_mines - flagged;

            // All hidden neighbors must be mines.
            if remaining > 0 && remaining == hidden.len() as u8 {
                return Some(Move::Flag(hidden[0].0, hidden[0].1));
            }

            // All mines accounted for — hidden neighbors are safe.
            if remaining == 0 && !hidden.is_empty() {
                return Some(Move::Reveal(hidden[0].0, hidden[0].1));
            }
        }
    }
    None
}

/// Build a constraint for each frontier numbered cell, then compare pairs.
/// If constraint B's cells are a subset of A's cells:
///   - If A.mines == B.mines → (A.cells - B.cells) are all safe.
///   - If A.mines - B.mines == |A.cells - B.cells| → (A.cells - B.cells) are all mines.
fn constraint_subset_deduction(board: &Board) -> Option<Move> {
    let constraints = build_constraints(board);

    for i in 0..constraints.len() {
        for j in 0..constraints.len() {
            if i == j {
                continue;
            }
            let a = &constraints[i];
            let b = &constraints[j];

            // Check if B ⊆ A.
            if !is_subset(&b.cells, &a.cells) {
                continue;
            }

            let diff: Vec<_> = a
                .cells
                .iter()
                .copied()
                .filter(|c| !b.cells.contains(c))
                .collect();

            if diff.is_empty() {
                continue;
            }

            let mine_diff = a.mines.saturating_sub(b.mines);

            // Same mine count → the difference cells are all safe.
            if a.mines == b.mines {
                return Some(Move::Reveal(diff[0].0, diff[0].1));
            }

            // Mine difference equals size of diff → all diff cells are mines.
            if mine_diff == diff.len() as u8 {
                return Some(Move::Flag(diff[0].0, diff[0].1));
            }
        }
    }

    None
}

/// Build one constraint per revealed numbered cell that still has hidden neighbors.
fn build_constraints(board: &Board) -> Vec<Constraint> {
    let mut constraints = Vec::new();

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
                .into_iter()
                .filter(|&(nx, ny)| board.cell(nx, ny).state == CellState::Hidden)
                .collect();

            let remaining = cell.adjacent_mines.saturating_sub(flagged);

            if !hidden.is_empty() && remaining > 0 {
                constraints.push(Constraint {
                    mines: remaining,
                    cells: hidden,
                });
            }
        }
    }

    constraints
}

fn is_subset(small: &[(usize, usize)], big: &[(usize, usize)]) -> bool {
    small.iter().all(|c| big.contains(c))
}

/// When no logical deduction is possible, estimate per-cell mine probability
/// from adjacent constraints and pick the cell least likely to be a mine.
/// Prefers frontier cells (adjacent to revealed) over interior unknowns,
/// since they open up more deduction opportunities.
fn smart_guess(board: &Board) -> Option<Move> {
    let mut hidden: Vec<(usize, usize)> = Vec::new();
    for y in 0..board.height {
        for x in 0..board.width {
            if board.cell(x, y).state == CellState::Hidden {
                hidden.push((x, y));
            }
        }
    }

    if hidden.is_empty() {
        return None;
    }

    let constraints = build_constraints(board);

    // For each hidden cell, track the maximum mine probability any single
    // constraint assigns to it. If no constraint touches a cell, use the
    // global background probability (remaining mines / remaining hidden cells).
    let remaining_mines = board.mine_count - board.flags_placed;
    let background_prob = if hidden.is_empty() {
        1.0
    } else {
        remaining_mines as f64 / hidden.len() as f64
    };

    let mut best: Option<((usize, usize), f64, bool)> = None; // (cell, prob, is_frontier)

    for &(x, y) in &hidden {
        // A cell is on the frontier if it neighbors at least one revealed cell.
        let on_frontier = neighbors_of(board, x, y)
            .iter()
            .any(|&(nx, ny)| board.cell(nx, ny).state == CellState::Revealed);

        // Compute max probability from all constraints that include this cell.
        let mut max_prob: Option<f64> = None;
        for c in &constraints {
            if c.cells.contains(&(x, y)) {
                let p = c.mines as f64 / c.cells.len() as f64;
                max_prob = Some(match max_prob {
                    Some(prev) => prev.max(p),
                    None => p,
                });
            }
        }

        let prob = max_prob.unwrap_or(background_prob);

        let is_better = match best {
            None => true,
            Some((_, best_prob, best_frontier)) => {
                // Prefer frontier cells, then lower probability.
                (on_frontier && !best_frontier)
                    || (on_frontier == best_frontier && prob < best_prob)
            }
        };

        if is_better {
            best = Some(((x, y), prob, on_frontier));
        }
    }

    let ((bx, by), _, _) = best?;
    Some(Move::Reveal(bx, by))
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
