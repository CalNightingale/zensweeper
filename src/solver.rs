use std::collections::HashMap;

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

    // Opening move: pick a corner for maximum flood-fill cascade.
    if board.cells_revealed == 0 {
        return Some(Move::Reveal(0, 0));
    }

    // Try single-cell deductions first (cheap).
    if let Some(mv) = single_cell_deduction(board) {
        return Some(mv);
    }

    // Build constraints and try subset reasoning.
    if let Some(mv) = constraint_subset_deduction(board) {
        return Some(mv);
    }

    // Global mine count deduction.
    if let Some(mv) = global_mine_deduction(board) {
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

/// If all mines are flagged, remaining hidden cells are safe.
/// If remaining hidden cells equals remaining mines, they're all mines.
fn global_mine_deduction(board: &Board) -> Option<Move> {
    let remaining_mines = board.mine_count - board.flags_placed;
    let mut first_hidden = None;
    let mut hidden_count = 0usize;

    for y in 0..board.height {
        for x in 0..board.width {
            if board.cell(x, y).state == CellState::Hidden {
                hidden_count += 1;
                if first_hidden.is_none() {
                    first_hidden = Some((x, y));
                }
            }
        }
    }

    let (x, y) = first_hidden?;

    if remaining_mines == 0 {
        return Some(Move::Reveal(x, y));
    }
    if remaining_mines == hidden_count {
        return Some(Move::Flag(x, y));
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

/// Enumerate valid mine configurations across coupled constraints to compute
/// exact per-cell mine probabilities. Cells with probability 0 are revealed,
/// cells with probability 1 are flagged, otherwise the safest cell is guessed.
fn smart_guess(board: &Board) -> Option<Move> {
    use std::collections::HashSet;

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

    let remaining_mines = board.mine_count - board.flags_placed;
    let constraints = build_constraints(board);

    // Collect all frontier cells (cells appearing in any constraint).
    let mut frontier_set: HashSet<(usize, usize)> = HashSet::new();
    for c in &constraints {
        for &cell in &c.cells {
            frontier_set.insert(cell);
        }
    }
    let frontier: Vec<(usize, usize)> = frontier_set.iter().copied().collect();
    let non_frontier_count = hidden.len() - frontier.len();
    let n = frontier.len();

    if n == 0 {
        // No frontier — all hidden cells are interior. Pick any.
        return Some(Move::Reveal(hidden[0].0, hidden[0].1));
    }

    // Map frontier cells to indices 0..n.
    let cell_idx: HashMap<(usize, usize), usize> =
        frontier.iter().enumerate().map(|(i, &c)| (c, i)).collect();

    // Index constraints by frontier cell indices.
    let indexed_constraints: Vec<(u8, Vec<usize>)> = constraints
        .iter()
        .map(|c| {
            let idxs: Vec<usize> = c.cells.iter().map(|cell| cell_idx[cell]).collect();
            (c.mines, idxs)
        })
        .collect();

    // Group frontier cells into connected components via union-find.
    let mut uf_parent: Vec<usize> = (0..n).collect();
    for (_, idxs) in &indexed_constraints {
        for i in 1..idxs.len() {
            uf_union(&mut uf_parent, idxs[0], idxs[i]);
        }
    }
    let mut comp_map: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        comp_map.entry(uf_find(&mut uf_parent, i)).or_default().push(i);
    }
    let components: Vec<Vec<usize>> = comp_map.into_values().collect();

    const MAX_COMPONENT_SIZE: usize = 30;
    let mut comp_results: Vec<ComponentResult> = Vec::new();

    for comp_cells in &components {
        let cn = comp_cells.len();

        if cn > MAX_COMPONENT_SIZE {
            // Fallback: treat as single uniform-density component.
            // Not ideal, but avoids exponential blowup.
            let mut totals = HashMap::new();
            let mut cell_mines = vec![HashMap::new(); cn];
            // Use average constraint density as rough estimate.
            totals.insert(0, 1.0); // dummy
            for (li, &ci) in comp_cells.iter().enumerate() {
                let mut max_p = 0.0f64;
                for (mines, idxs) in &indexed_constraints {
                    if idxs.contains(&ci) {
                        max_p = max_p.max(*mines as f64 / idxs.len() as f64);
                    }
                }
                cell_mines[li].insert(0, max_p);
            }
            comp_results.push(ComponentResult {
                global_indices: comp_cells.clone(),
                totals,
                cell_mines,
            });
            continue;
        }

        // Build local constraints for this component.
        let comp_set: HashSet<usize> = comp_cells.iter().copied().collect();
        let local_idx: HashMap<usize, usize> =
            comp_cells.iter().enumerate().map(|(li, &ci)| (ci, li)).collect();

        let local_constraints: Vec<(u8, Vec<usize>)> = indexed_constraints
            .iter()
            .filter(|(_, idxs)| idxs.iter().any(|i| comp_set.contains(i)))
            .map(|(mines, idxs)| {
                (*mines, idxs.iter().filter_map(|i| local_idx.get(i).copied()).collect())
            })
            .collect();

        let mut cell_mines: Vec<HashMap<usize, f64>> = vec![HashMap::new(); cn];
        let mut totals: HashMap<usize, f64> = HashMap::new();
        let mut assignment: Vec<Option<bool>> = vec![None; cn];

        enumerate_configs(
            0, &mut assignment, &local_constraints,
            &mut cell_mines, &mut totals, 0,
        );

        comp_results.push(ComponentResult {
            global_indices: comp_cells.clone(),
            totals,
            cell_mines,
        });
    }

    // Cross-component enumeration weighted by global mine constraint.
    // Precompute binomial coefficients C(non_frontier_count, k) for k = 0..remaining_mines.
    let comb = precompute_comb(non_frontier_count, remaining_mines);

    let mut weighted_mine: Vec<f64> = vec![0.0; n];
    let mut total_weight: f64 = 0.0;

    // Recursively enumerate all valid combinations of per-component mine counts.
    let mut chosen_k: Vec<usize> = vec![0; comp_results.len()];
    cross_enumerate(
        0, &comp_results, 0, 1.0,
        remaining_mines, non_frontier_count, &comb,
        &mut weighted_mine, &mut total_weight, &mut chosen_k,
    );

    if total_weight <= 0.0 {
        return Some(Move::Reveal(hidden[0].0, hidden[0].1));
    }

    // Check for deterministic cells (probability 0 or 1).
    for (gi, &cell) in frontier.iter().enumerate() {
        let prob = weighted_mine[gi] / total_weight;
        if prob < 1e-9 {
            return Some(Move::Reveal(cell.0, cell.1));
        }
        if prob > 1.0 - 1e-9 {
            return Some(Move::Flag(cell.0, cell.1));
        }
    }

    // Non-frontier mine probability.
    let expected_frontier_mines: f64 = weighted_mine.iter().sum::<f64>() / total_weight;
    let nf_prob = if non_frontier_count > 0 {
        (remaining_mines as f64 - expected_frontier_mines) / non_frontier_count as f64
    } else {
        1.0
    };

    // Pick safest cell, preferring frontier for information gain.
    let mut best: Option<((usize, usize), f64, bool)> = None;

    for (gi, &cell) in frontier.iter().enumerate() {
        let prob = weighted_mine[gi] / total_weight;
        let is_better = match best {
            None => true,
            Some((_, bp, bf)) => (true && !bf) || (bf && prob < bp),
        };
        if is_better {
            best = Some((cell, prob, true));
        }
    }
    for &(x, y) in &hidden {
        if frontier_set.contains(&(x, y)) {
            continue;
        }
        let is_better = match best {
            None => true,
            Some((_, bp, bf)) => !bf && nf_prob < bp,
        };
        if is_better {
            best = Some(((x, y), nf_prob, false));
        }
    }

    let ((bx, by), _, _) = best?;
    Some(Move::Reveal(bx, by))
}

/// Backtracking enumeration of valid mine assignments for a connected component.
fn enumerate_configs(
    idx: usize,
    assignment: &mut Vec<Option<bool>>,
    constraints: &[(u8, Vec<usize>)],
    cell_mines: &mut Vec<HashMap<usize, f64>>,
    totals: &mut HashMap<usize, f64>,
    current_mines: usize,
) {
    let n = assignment.len();
    if idx == n {
        for (mines, idxs) in constraints {
            let count: u8 = idxs.iter().filter(|&&i| assignment[i] == Some(true)).count() as u8;
            if count != *mines {
                return;
            }
        }
        *totals.entry(current_mines).or_insert(0.0) += 1.0;
        for i in 0..n {
            if assignment[i] == Some(true) {
                *cell_mines[i].entry(current_mines).or_insert(0.0) += 1.0;
            }
        }
        return;
    }

    // Pruning.
    for (mines, idxs) in constraints {
        let assigned_mines: u8 = idxs.iter().filter(|&&i| assignment[i] == Some(true)).count() as u8;
        let unassigned: u8 = idxs.iter().filter(|&&i| assignment[i].is_none()).count() as u8;
        if assigned_mines > *mines || *mines - assigned_mines > unassigned {
            return;
        }
    }

    assignment[idx] = Some(false);
    enumerate_configs(idx + 1, assignment, constraints, cell_mines, totals, current_mines);
    assignment[idx] = Some(true);
    enumerate_configs(idx + 1, assignment, constraints, cell_mines, totals, current_mines + 1);
    assignment[idx] = None;
}

/// Union-find helpers.
fn uf_find(parent: &mut [usize], x: usize) -> usize {
    if parent[x] != x {
        parent[x] = uf_find(parent, parent[x]);
    }
    parent[x]
}
fn uf_union(parent: &mut [usize], a: usize, b: usize) {
    let ra = uf_find(parent, a);
    let rb = uf_find(parent, b);
    if ra != rb {
        parent[ra] = rb;
    }
}

/// Precompute C(n, k) as f64 for n up to `max_n` and k up to `max_k`.
fn precompute_comb(max_n: usize, max_k: usize) -> Vec<Vec<f64>> {
    let rows = max_n + 1;
    let cols = max_k + 1;
    let mut c = vec![vec![0.0f64; cols]; rows];
    for i in 0..rows {
        c[i][0] = 1.0;
        for j in 1..cols.min(i + 1) {
            c[i][j] = c[i - 1][j - 1] + c[i - 1][j];
        }
    }
    c
}

/// Recursively enumerate all valid per-component mine count combinations,
/// weighting by binomial coefficient for non-frontier cells.
fn cross_enumerate(
    comp_idx: usize,
    comp_results: &[ComponentResult],
    frontier_mines: usize,
    weight: f64,
    remaining_mines: usize,
    non_frontier_count: usize,
    comb: &[Vec<f64>],
    weighted_mine: &mut [f64],
    total_weight: &mut f64,
    chosen_k: &mut Vec<usize>,
) {
    if comp_idx == comp_results.len() {
        if frontier_mines > remaining_mines {
            return;
        }
        let nf_mines = remaining_mines - frontier_mines;
        if nf_mines > non_frontier_count {
            return;
        }
        let nf_weight = comb[non_frontier_count][nf_mines];
        let w = weight * nf_weight;
        *total_weight += w;

        for (ci, cr) in comp_results.iter().enumerate() {
            let k = chosen_k[ci];
            let comp_total = cr.totals[&k];
            for (li, gi) in cr.global_indices.iter().enumerate() {
                if let Some(&count) = cr.cell_mines[li].get(&k) {
                    weighted_mine[*gi] += w * count / comp_total;
                }
            }
        }
        return;
    }

    for (&k, &config_count) in &comp_results[comp_idx].totals {
        if frontier_mines + k > remaining_mines {
            continue;
        }
        chosen_k[comp_idx] = k;
        cross_enumerate(
            comp_idx + 1, comp_results, frontier_mines + k,
            weight * config_count, remaining_mines, non_frontier_count,
            comb, weighted_mine, total_weight, chosen_k,
        );
    }
}

struct ComponentResult {
    global_indices: Vec<usize>,
    totals: HashMap<usize, f64>,
    cell_mines: Vec<HashMap<usize, f64>>,
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
