# Minesweeper

Classic Minesweeper for the terminal, written in Rust.

Play it yourself or watch the built-in AI solver play in zen mode.

## Features

- Three difficulty levels: Beginner (9x9), Intermediate (16x16), Expert (30x16)
- Keyboard controls: arrow keys, WASD, or vim-style hjkl
- First click is always safe
- Flood-fill reveal on empty cells
- Classic number color scheme
- **Zen mode**: sit back and watch an AI solver play through expert games on loop

## Install

Requires [Rust](https://rustup.rs/).

```bash
git clone <repo-url>
cd minesweeper
cargo build --release
```

## Usage

```bash
# Interactive mode
cargo run --release

# Zen mode — AI plays expert continuously
cargo run --release -- --zen
```

### Controls

| Key | Action |
|-----|--------|
| Arrow keys / WASD / hjkl | Move cursor |
| Space / Enter | Reveal cell |
| F | Toggle flag |
| R | Restart |
| Q / Esc | Quit |

## Zen Mode

The `--zen` flag starts an automated solver that plays expert Minesweeper in a loop. You can benchmark it headlessly with `cargo run --release -- --bench 5000`.

### How the solver thinks

Each turn, the solver runs through a decision cascade, stopping as soon as it finds a move:

1. **Opening move** — always reveals a corner cell to maximize the initial flood-fill cascade.

2. **Single-cell deductions** — for each revealed number, counts its hidden and flagged neighbors. If hidden neighbors equals remaining mines, they're all mines (flag one). If remaining mines is zero, they're all safe (reveal one). This is the cheapest check and handles most moves.

3. **Constraint subset reasoning** — builds a constraint for each frontier number: "exactly *k* of these hidden neighbors are mines." Compares every pair of constraints; if one is a subset of another, the difference cells can sometimes be determined as all-safe or all-mines. This catches patterns that single-cell logic misses.

4. **Global mine count** — compares total remaining mines to total remaining hidden cells. If they're equal, every hidden cell is a mine. If zero mines remain, every hidden cell is safe. Cleans up endgame positions.

5. **Constraint enumeration** — when no deterministic move exists, the solver groups frontier cells into connected components via union-find, then enumerates every valid mine assignment for each component using backtracking with constraint pruning. Cross-component combinations are weighted by the binomial coefficient C(*non-frontier cells*, *remaining mines*) to account for the global mine count. This produces exact per-cell mine probabilities. Any cell with probability 0 or 1 is handled deterministically (this subsumes the pairwise subset reasoning for coupled constraints). Otherwise, the solver reveals the cell with the lowest mine probability, preferring frontier cells for information gain.

### Win rate

On Expert (30x16, 99 mines) the solver wins about **40%** of games — near the theoretical ceiling for this difficulty. Most losses come from forced 50/50 guesses in the endgame.

The speed is configurable via `ZEN_INPUTS_PER_SEC` in `src/main.rs`.
