/// Shared settings for both terminal and web versions.

/// RGB color tuple.
pub type Rgb = (u8, u8, u8);

/// Colors for numbers 1-8 (classic Microsoft Minesweeper palette).
/// Index 0 is unused — access as `NUMBER_COLORS[n]` for number `n`.
pub const NUMBER_COLORS: [Rgb; 9] = [
    (0, 0, 0),           // 0: unused
    (0, 1, 254),         // 1: blue
    (0, 130, 2),         // 2: green
    (254, 0, 0),         // 3: red
    (1, 0, 130),         // 4: dark blue
    (132, 0, 2),         // 5: maroon
    (0, 130, 130),       // 6: teal
    (132, 1, 133),       // 7: purple
    (115, 115, 115),     // 8: gray
];

/// Difficulty presets: (width, height, mines).
pub const PRESET_BEGINNER: (usize, usize, usize) = (9, 9, 10);
pub const PRESET_INTERMEDIATE: (usize, usize, usize) = (16, 16, 40);
pub const PRESET_EXPERT: (usize, usize, usize) = (30, 16, 99);

pub const PRESETS: [(usize, usize, usize); 3] = [
    PRESET_BEGINNER,
    PRESET_INTERMEDIATE,
    PRESET_EXPERT,
];

/// Menu labels for each difficulty.
pub const MENU_OPTIONS: [&str; 3] = [
    "Beginner     (9 x 9,   10 mines)",
    "Intermediate (16 x 16, 40 mines)",
    "Expert       (30 x 16, 99 mines)",
];

/// Unicode symbols for cell display.
pub const SYMBOL_HIDDEN: char = '\u{25A0}'; // ■
pub const SYMBOL_FLAG: char = '\u{2691}';   // ⚑
pub const SYMBOL_MINE: char = '\u{2739}';   // ✹

/// Background color for the game area.
pub const BG_COLOR: Rgb = (30, 30, 30);

/// Characters per cell in the grid.
pub const CELL_WIDTH: usize = 3;

/// Zen mode speed: inputs per second.
pub const ZEN_INPUTS_PER_SEC: f64 = 10.0;

/// Countdown seconds between zen mode games.
pub const ZEN_END_COUNTDOWN: u32 = 3;
