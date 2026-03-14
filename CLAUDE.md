# Minesweeper

Terminal-based Minesweeper in Rust with interactive and automated (zen) modes.

## Build & Run

```bash
cargo build
cargo run                  # interactive mode — menu selects difficulty
cargo run -- --zen         # zen mode — AI solver plays expert continuously
```

## Architecture

Single binary crate, 6 modules:

- `main.rs` — entry point, terminal lifecycle (`TerminalGuard`), game loops (interactive + zen)
- `board.rs` — `Board` struct, mine placement (deferred to first reveal), BFS flood-fill, win/loss
- `cell.rs` — `Cell`, `CellState` enum
- `input.rs` — maps crossterm key events to `Action` enum
- `render.rs` — draws grid, menu, status bar, win/loss messages via crossterm `queue!`
- `solver.rs` — zen mode AI: single-cell deductions → constraint subset reasoning → probabilistic guess

## Dependencies

Only two: `crossterm` (terminal UI) and `rand` (mine placement).

## Key Constants

In `main.rs`:
- `ZEN_INPUTS_PER_SEC` — speed of zen mode (inputs/sec, covers arrow keys + actions)
- `ZEN_END_COUNTDOWN` — seconds to pause between zen games

## Conventions

- Flat `Vec<Cell>` with row-major indexing (`y * width + x`), not nested vecs
- `TerminalGuard` with `Drop` ensures terminal state is always restored
- Zen mode uses `poll_quit()` with `Instant`-based deadlines to drain held keys without speeding up
- RGB colors for numbers 1-8 matching classic Microsoft Minesweeper palette
- Unicode symbols: `■` hidden, `⚑` flag, `✹` mine
