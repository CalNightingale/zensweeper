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

The `--zen` flag starts an automated solver that plays expert Minesweeper in a loop. The solver uses three tiers of strategy:

1. **Single-cell deductions** — trivial flag/reveal based on numbered cells
2. **Constraint subset reasoning** — compares overlapping constraints between neighboring numbered cells to deduce safe cells and mines
3. **Probabilistic guessing** — when no logical move exists, picks the frontier cell with the lowest estimated mine probability

The speed is configurable via `ZEN_INPUTS_PER_SEC` in `src/main.rs`.
