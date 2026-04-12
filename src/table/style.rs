use crate::color::Color;

use super::ANSI_RESET;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StyleAction {
    Color(Color),
    OnColor(Color),
    Clear,
    Normal,
    Bold,
    Dimmed,
    Italic,
    Underline,
    Blink,
    Reversed,
    Hidden,
    Strikethrough,
}

macro_rules! impl_style_methods {
    ($ty:ty, $push:expr) => {
        impl $ty {
            fn push_style(self, action: $crate::table::style::StyleAction) -> Self {
                ($push)(self, action)
            }

            /// Apply a foreground color.
            pub fn color(self, color: $crate::color::Color) -> Self {
                self.push_style($crate::table::style::StyleAction::Color(color))
            }

            /// Apply a background color.
            pub fn on_color(self, color: $crate::color::Color) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(color))
            }

            /// Clear all formatting.
            pub fn clear(self) -> Self {
                self.push_style($crate::table::style::StyleAction::Clear)
            }

            /// Reset formatting back to normal.
            pub fn normal(self) -> Self {
                self.push_style($crate::table::style::StyleAction::Normal)
            }

            /// Make the text bold.
            pub fn bold(self) -> Self {
                self.push_style($crate::table::style::StyleAction::Bold)
            }

            /// Make the text dimmed.
            pub fn dimmed(self) -> Self {
                self.push_style($crate::table::style::StyleAction::Dimmed)
            }

            /// Make the text italic.
            pub fn italic(self) -> Self {
                self.push_style($crate::table::style::StyleAction::Italic)
            }

            /// Underline the text.
            pub fn underline(self) -> Self {
                self.push_style($crate::table::style::StyleAction::Underline)
            }

            /// Blink the text.
            pub fn blink(self) -> Self {
                self.push_style($crate::table::style::StyleAction::Blink)
            }

            /// Reverse foreground and background colors.
            pub fn reversed(self) -> Self {
                self.push_style($crate::table::style::StyleAction::Reversed)
            }

            /// Hide the text.
            pub fn hidden(self) -> Self {
                self.push_style($crate::table::style::StyleAction::Hidden)
            }

            /// Strike the text through.
            pub fn strikethrough(self) -> Self {
                self.push_style($crate::table::style::StyleAction::Strikethrough)
            }

            /// Use a black foreground color.
            pub fn black(self) -> Self {
                self.color($crate::color::Color::Black)
            }

            /// Use a red foreground color.
            pub fn red(self) -> Self {
                self.color($crate::color::Color::Red)
            }

            /// Use a green foreground color.
            pub fn green(self) -> Self {
                self.color($crate::color::Color::Green)
            }

            /// Use a yellow foreground color.
            pub fn yellow(self) -> Self {
                self.color($crate::color::Color::Yellow)
            }

            /// Use a blue foreground color.
            pub fn blue(self) -> Self {
                self.color($crate::color::Color::Blue)
            }

            /// Use a magenta foreground color.
            pub fn magenta(self) -> Self {
                self.color($crate::color::Color::Magenta)
            }

            /// Use a cyan foreground color.
            pub fn cyan(self) -> Self {
                self.color($crate::color::Color::Cyan)
            }

            /// Use a white foreground color.
            pub fn white(self) -> Self {
                self.color($crate::color::Color::White)
            }

            /// Use a bright black foreground color.
            pub fn bright_black(self) -> Self {
                self.color($crate::color::Color::BrightBlack)
            }

            /// Use a bright red foreground color.
            pub fn bright_red(self) -> Self {
                self.color($crate::color::Color::BrightRed)
            }

            /// Use a bright green foreground color.
            pub fn bright_green(self) -> Self {
                self.color($crate::color::Color::BrightGreen)
            }

            /// Use a bright yellow foreground color.
            pub fn bright_yellow(self) -> Self {
                self.color($crate::color::Color::BrightYellow)
            }

            /// Use a bright blue foreground color.
            pub fn bright_blue(self) -> Self {
                self.color($crate::color::Color::BrightBlue)
            }

            /// Use a bright magenta foreground color.
            pub fn bright_magenta(self) -> Self {
                self.color($crate::color::Color::BrightMagenta)
            }

            /// Use a bright cyan foreground color.
            pub fn bright_cyan(self) -> Self {
                self.color($crate::color::Color::BrightCyan)
            }

            /// Use a bright white foreground color.
            pub fn bright_white(self) -> Self {
                self.color($crate::color::Color::BrightWhite)
            }

            /// Use a purple foreground color.
            pub fn purple(self) -> Self {
                self.color($crate::color::Color::Magenta)
            }

            /// Use a bright purple foreground color.
            pub fn bright_purple(self) -> Self {
                self.color($crate::color::Color::BrightMagenta)
            }

            /// Use an ANSI 8-bit foreground color.
            pub fn ansi_color(self, color: impl Into<u8>) -> Self {
                self.push_style($crate::table::style::StyleAction::Color(
                    $crate::color::Color::AnsiColor(color.into()),
                ))
            }

            /// Use a truecolor foreground color.
            pub fn truecolor(self, red: u8, green: u8, blue: u8) -> Self {
                self.push_style($crate::table::style::StyleAction::Color(
                    $crate::color::Color::TrueColor {
                        r: red,
                        g: green,
                        b: blue,
                    },
                ))
            }

            /// Use a black background color.
            pub fn on_black(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::Black,
                ))
            }

            /// Use a red background color.
            pub fn on_red(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::Red,
                ))
            }

            /// Use a green background color.
            pub fn on_green(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::Green,
                ))
            }

            /// Use a yellow background color.
            pub fn on_yellow(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::Yellow,
                ))
            }

            /// Use a blue background color.
            pub fn on_blue(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::Blue,
                ))
            }

            /// Use a magenta background color.
            pub fn on_magenta(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::Magenta,
                ))
            }

            /// Use a cyan background color.
            pub fn on_cyan(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::Cyan,
                ))
            }

            /// Use a white background color.
            pub fn on_white(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::White,
                ))
            }

            /// Use a bright black background color.
            pub fn on_bright_black(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::BrightBlack,
                ))
            }

            /// Use a bright red background color.
            pub fn on_bright_red(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::BrightRed,
                ))
            }

            /// Use a bright green background color.
            pub fn on_bright_green(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::BrightGreen,
                ))
            }

            /// Use a bright yellow background color.
            pub fn on_bright_yellow(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::BrightYellow,
                ))
            }

            /// Use a bright blue background color.
            pub fn on_bright_blue(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::BrightBlue,
                ))
            }

            /// Use a bright magenta background color.
            pub fn on_bright_magenta(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::BrightMagenta,
                ))
            }

            /// Use a bright cyan background color.
            pub fn on_bright_cyan(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::BrightCyan,
                ))
            }

            /// Use a bright white background color.
            pub fn on_bright_white(self) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::BrightWhite,
                ))
            }

            /// Use an ANSI 8-bit background color.
            pub fn custom_color(self, color: impl Into<$crate::color::CustomColor>) -> Self {
                let color = color.into();
                self.push_style($crate::table::style::StyleAction::Color(
                    $crate::color::Color::TrueColor {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                    },
                ))
            }

            /// Use a custom truecolor background color.
            pub fn on_custom_color(self, color: impl Into<$crate::color::CustomColor>) -> Self {
                let color = color.into();
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::TrueColor {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                    },
                ))
            }

            /// Use an ANSI 8-bit background color.
            pub fn on_ansi_color(self, color: impl Into<u8>) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::AnsiColor(color.into()),
                ))
            }

            /// Use a truecolor background color.
            pub fn on_truecolor(self, red: u8, green: u8, blue: u8) -> Self {
                self.push_style($crate::table::style::StyleAction::OnColor(
                    $crate::color::Color::TrueColor {
                        r: red,
                        g: green,
                        b: blue,
                    },
                ))
            }
        }
    };
}

pub(super) use impl_style_methods;

pub(super) fn apply_style_actions(content: &str, actions: &[StyleAction]) -> String {
    if content.is_empty() || actions.is_empty() {
        return content.to_string();
    }

    let mut prefix = String::new();
    let mut has_style = false;

    for action in actions {
        match action {
            StyleAction::Color(color) => {
                prefix.push_str(&color.ansi_fg());
                has_style = true;
            }
            StyleAction::OnColor(color) => {
                prefix.push_str(&color.ansi_bg());
                has_style = true;
            }
            StyleAction::Clear | StyleAction::Normal => {
                prefix.clear();
                has_style = false;
            }
            StyleAction::Bold => {
                prefix.push_str("\x1b[1m");
                has_style = true;
            }
            StyleAction::Dimmed => {
                prefix.push_str("\x1b[2m");
                has_style = true;
            }
            StyleAction::Italic => {
                prefix.push_str("\x1b[3m");
                has_style = true;
            }
            StyleAction::Underline => {
                prefix.push_str("\x1b[4m");
                has_style = true;
            }
            StyleAction::Blink => {
                prefix.push_str("\x1b[5m");
                has_style = true;
            }
            StyleAction::Reversed => {
                prefix.push_str("\x1b[7m");
                has_style = true;
            }
            StyleAction::Hidden => {
                prefix.push_str("\x1b[8m");
                has_style = true;
            }
            StyleAction::Strikethrough => {
                prefix.push_str("\x1b[9m");
                has_style = true;
            }
        }
    }

    if !has_style {
        return content.to_string();
    }

    format!("{}{}{}", prefix, content, ANSI_RESET)
}
