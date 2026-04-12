/// A color used for foreground and background text styling.
///
/// Named variants map to the standard 16 ANSI terminal colors. [`AnsiColor`]
/// selects one of the 256 extended palette entries. [`TrueColor`] accepts
/// arbitrary 24-bit RGB values when the terminal supports it.
///
/// [`AnsiColor`]: Color::AnsiColor
/// [`TrueColor`]: Color::TrueColor
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Color {
    /// Standard black (ANSI 30 / 40).
    Black,
    /// Standard red (ANSI 31 / 41).
    Red,
    /// Standard green (ANSI 32 / 42).
    Green,
    /// Standard yellow (ANSI 33 / 43).
    Yellow,
    /// Standard blue (ANSI 34 / 44).
    Blue,
    /// Standard magenta (ANSI 35 / 45).
    Magenta,
    /// Standard cyan (ANSI 36 / 46).
    Cyan,
    /// Standard white (ANSI 37 / 47).
    White,
    /// Bright black / dark gray (ANSI 90 / 100).
    BrightBlack,
    /// Bright red (ANSI 91 / 101).
    BrightRed,
    /// Bright green (ANSI 92 / 102).
    BrightGreen,
    /// Bright yellow (ANSI 93 / 103).
    BrightYellow,
    /// Bright blue (ANSI 94 / 104).
    BrightBlue,
    /// Bright magenta (ANSI 95 / 105).
    BrightMagenta,
    /// Bright cyan (ANSI 96 / 106).
    BrightCyan,
    /// Bright white (ANSI 97 / 107).
    BrightWhite,
    /// One of the 256-color extended palette entries (`\x1b[38;5;n`).
    AnsiColor(u8),
    /// A 24-bit truecolor value (`\x1b[38;2;r;g;b`).
    TrueColor {
        /// Red component (0–255).
        r: u8,
        /// Green component (0–255).
        g: u8,
        /// Blue component (0–255).
        b: u8,
    },
}

impl Color {
    /// Return the ANSI escape sequence for this color used as a foreground.
    pub(crate) fn ansi_fg(self) -> String {
        match self {
            Self::Black => "\x1b[30m".to_string(),
            Self::Red => "\x1b[31m".to_string(),
            Self::Green => "\x1b[32m".to_string(),
            Self::Yellow => "\x1b[33m".to_string(),
            Self::Blue => "\x1b[34m".to_string(),
            Self::Magenta => "\x1b[35m".to_string(),
            Self::Cyan => "\x1b[36m".to_string(),
            Self::White => "\x1b[37m".to_string(),
            Self::BrightBlack => "\x1b[90m".to_string(),
            Self::BrightRed => "\x1b[91m".to_string(),
            Self::BrightGreen => "\x1b[92m".to_string(),
            Self::BrightYellow => "\x1b[93m".to_string(),
            Self::BrightBlue => "\x1b[94m".to_string(),
            Self::BrightMagenta => "\x1b[95m".to_string(),
            Self::BrightCyan => "\x1b[96m".to_string(),
            Self::BrightWhite => "\x1b[97m".to_string(),
            Self::AnsiColor(n) => format!("\x1b[38;5;{n}m"),
            Self::TrueColor { r, g, b } => format!("\x1b[38;2;{r};{g};{b}m"),
        }
    }

    /// Return the ANSI escape sequence for this color used as a background.
    pub(crate) fn ansi_bg(self) -> String {
        match self {
            Self::Black => "\x1b[40m".to_string(),
            Self::Red => "\x1b[41m".to_string(),
            Self::Green => "\x1b[42m".to_string(),
            Self::Yellow => "\x1b[43m".to_string(),
            Self::Blue => "\x1b[44m".to_string(),
            Self::Magenta => "\x1b[45m".to_string(),
            Self::Cyan => "\x1b[46m".to_string(),
            Self::White => "\x1b[47m".to_string(),
            Self::BrightBlack => "\x1b[100m".to_string(),
            Self::BrightRed => "\x1b[101m".to_string(),
            Self::BrightGreen => "\x1b[102m".to_string(),
            Self::BrightYellow => "\x1b[103m".to_string(),
            Self::BrightBlue => "\x1b[104m".to_string(),
            Self::BrightMagenta => "\x1b[105m".to_string(),
            Self::BrightCyan => "\x1b[106m".to_string(),
            Self::BrightWhite => "\x1b[107m".to_string(),
            Self::AnsiColor(n) => format!("\x1b[48;5;{n}m"),
            Self::TrueColor { r, g, b } => format!("\x1b[48;2;{r};{g};{b}m"),
        }
    }
}

/// A truecolor value specified by red, green, and blue byte components.
///
/// Primarily used with the [`custom_color`](crate::Cell::custom_color) and
/// [`on_custom_color`](crate::Cell::on_custom_color) methods.
pub struct CustomColor {
    /// Red component (0–255).
    pub r: u8,
    /// Green component (0–255).
    pub g: u8,
    /// Blue component (0–255).
    pub b: u8,
}

impl CustomColor {
    /// Create a new [`CustomColor`] from red, green, and blue byte components.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<(u8, u8, u8)> for CustomColor {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}
