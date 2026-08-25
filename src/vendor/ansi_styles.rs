//! ANSI escape codes for styling strings in the terminal.
//!
//! Port of the vendored `ansi-styles` module, extended — as the original was —
//! with the `SGR 4:x` underline substyles and the `SGR 58`/`59` underline
//! colours, neither of which exist upstream.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Offset added to a foreground SGR parameter to make it a background one.
const ANSI_BACKGROUND_OFFSET: u16 = 10;

/// Offset added to a foreground SGR parameter to make it an underline one.
const ANSI_UNDERLINE_OFFSET: u16 = 20;

/// The pair of ANSI terminal control sequences that begin and end a style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CsPair {
    /// The ANSI terminal control sequence for starting this style.
    pub open: &'static str,
    /// The ANSI terminal control sequence for ending this style.
    pub close: &'static str,
}

/// The three colour planes a terminal can address independently.
///
/// Each plane has its own family of SGR parameters, derived from the foreground
/// ones by a fixed offset — except for [`ColorType::Underline`]'s basic form,
/// which `SGR 58` does not provide at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorType {
    /// Text colour — `SGR 30`–`37`, `38`, `90`–`97`; closed by `SGR 39`.
    Foreground,
    /// Background colour — `SGR 40`–`47`, `48`, `100`–`107`; closed by `SGR 49`.
    Background,
    /// Underline colour — `SGR 58`; closed by `SGR 59`.
    Underline,
}

impl ColorType {
    /// The ANSI terminal control sequence for ending a colour of this plane.
    #[must_use]
    pub const fn close(self) -> &'static str {
        match self {
            Self::Foreground => "\u{1b}[39m",
            Self::Background => "\u{1b}[49m",
            Self::Underline => "\u{1b}[59m",
        }
    }

    /// The offset from the foreground SGR parameters to this plane's.
    const fn offset(self) -> u16 {
        match self {
            Self::Foreground => 0,
            Self::Background => ANSI_BACKGROUND_OFFSET,
            Self::Underline => ANSI_UNDERLINE_OFFSET,
        }
    }

    /// Wrap a basic 16-colour code (`30`–`37`, `90`–`97`) for this plane.
    ///
    /// `SGR 58` has no basic 16-colour form, so for [`ColorType::Underline`]
    /// the basic colour code is mapped to its palette index instead.
    #[must_use]
    pub fn ansi(self, code: u8) -> String {
        if matches!(self, Self::Underline) {
            let index = if code < 90 { code - 30 } else { code - 90 + 8 };
            return format!("\u{1b}[58;5;{index}m");
        }

        format!("\u{1b}[{}m", u16::from(code) + self.offset())
    }

    /// Wrap an 8-bit palette index for this plane.
    #[must_use]
    pub fn ansi256(self, code: u8) -> String {
        format!("\u{1b}[{};5;{code}m", 38 + self.offset())
    }

    /// Wrap a 24-bit colour for this plane.
    #[must_use]
    pub fn ansi16m(self, red: u8, green: u8, blue: u8) -> String {
        format!("\u{1b}[{};2;{red};{green};{blue}m", 38 + self.offset())
    }
}

/// The single source of truth for every named style.
///
/// Invoking this with the name of a callback macro replays the whole style
/// table into it, so that the escape sequences, the exported name lists, and
/// the style methods on [`Chalk`](crate::Chalk) and [`Style`](crate::Style) are
/// all generated from one table rather than kept in step by hand.
///
/// Each row is `method => "jsName", open_parameter, close_parameter, "doc"`.
/// The row order is the declaration order of the original object literal, and
/// it is observable: it is the order of the exported name lists.
macro_rules! with_style_table {
    ($callback:ident) => {
        $callback! {
            modifier {
                reset => "reset", 0, 0, "Modifier: Reset the current style.";
                // `21` isn't widely supported and `22` does the same thing.
                bold => "bold", 1, 22, "Modifier: Make the text bold.";
                dim => "dim", 2, 22, "Modifier: Make the text have lower opacity.";
                italic => "italic", 3, 23, "Modifier: Make the text italic. *(Not widely supported)*";
                underline => "underline", 4, 24, "Modifier: Put a horizontal line below the text. *(Not widely supported)*";
                // Extended underline styles (`SGR 4:x` sub-parameters). Not in upstream `ansi-styles`.
                underline_double => "underlineDouble", "4:2", 24, "Modifier: Put a double horizontal line below the text. *(Not widely supported)*";
                underline_curly => "underlineCurly", "4:3", 24, "Modifier: Put a curly horizontal line below the text. *(Not widely supported)*";
                underline_dotted => "underlineDotted", "4:4", 24, "Modifier: Put a dotted horizontal line below the text. *(Not widely supported)*";
                underline_dashed => "underlineDashed", "4:5", 24, "Modifier: Put a dashed horizontal line below the text. *(Not widely supported)*";
                overline => "overline", 53, 55, "Modifier: Put a horizontal line above the text. *(Not widely supported)*";
                inverse => "inverse", 7, 27, "Modifier: Invert background and foreground colors.";
                hidden => "hidden", 8, 28, "Modifier: Print the text but make it invisible.";
                strikethrough => "strikethrough", 9, 29, "Modifier: Put a horizontal line through the center of the text. *(Not widely supported)*";
            }
            color {
                black => "black", 30, 39, "Text colour: black.";
                red => "red", 31, 39, "Text colour: red.";
                green => "green", 32, 39, "Text colour: green.";
                yellow => "yellow", 33, 39, "Text colour: yellow.";
                blue => "blue", 34, 39, "Text colour: blue.";
                magenta => "magenta", 35, 39, "Text colour: magenta.";
                cyan => "cyan", 36, 39, "Text colour: cyan.";
                white => "white", 37, 39, "Text colour: white.";

                black_bright => "blackBright", 90, 39, "Text colour: bright black.";
                gray => "gray", 90, 39, "Text colour: alias for `black_bright`.";
                grey => "grey", 90, 39, "Text colour: alias for `black_bright`.";
                red_bright => "redBright", 91, 39, "Text colour: bright red.";
                green_bright => "greenBright", 92, 39, "Text colour: bright green.";
                yellow_bright => "yellowBright", 93, 39, "Text colour: bright yellow.";
                blue_bright => "blueBright", 94, 39, "Text colour: bright blue.";
                magenta_bright => "magentaBright", 95, 39, "Text colour: bright magenta.";
                cyan_bright => "cyanBright", 96, 39, "Text colour: bright cyan.";
                white_bright => "whiteBright", 97, 39, "Text colour: bright white.";
            }
            bg_color {
                bg_black => "bgBlack", 40, 49, "Background colour: black.";
                bg_red => "bgRed", 41, 49, "Background colour: red.";
                bg_green => "bgGreen", 42, 49, "Background colour: green.";
                bg_yellow => "bgYellow", 43, 49, "Background colour: yellow.";
                bg_blue => "bgBlue", 44, 49, "Background colour: blue.";
                bg_magenta => "bgMagenta", 45, 49, "Background colour: magenta.";
                bg_cyan => "bgCyan", 46, 49, "Background colour: cyan.";
                bg_white => "bgWhite", 47, 49, "Background colour: white.";

                bg_black_bright => "bgBlackBright", 100, 49, "Background colour: bright black.";
                bg_gray => "bgGray", 100, 49, "Background colour: alias for `bg_black_bright`.";
                bg_grey => "bgGrey", 100, 49, "Background colour: alias for `bg_black_bright`.";
                bg_red_bright => "bgRedBright", 101, 49, "Background colour: bright red.";
                bg_green_bright => "bgGreenBright", 102, 49, "Background colour: bright green.";
                bg_yellow_bright => "bgYellowBright", 103, 49, "Background colour: bright yellow.";
                bg_blue_bright => "bgBlueBright", 104, 49, "Background colour: bright blue.";
                bg_magenta_bright => "bgMagentaBright", 105, 49, "Background colour: bright magenta.";
                bg_cyan_bright => "bgCyanBright", 106, 49, "Background colour: bright cyan.";
                bg_white_bright => "bgWhiteBright", 107, 49, "Background colour: bright white.";
            }
            underline_color {
                // `SGR 58` has no basic 16-colour form, so every named underline
                // colour is spelled as its 256-colour palette index.
                underline_black => "underlineBlack", "58;5;0", 59, "Underline colour: black.";
                underline_red => "underlineRed", "58;5;1", 59, "Underline colour: red.";
                underline_green => "underlineGreen", "58;5;2", 59, "Underline colour: green.";
                underline_yellow => "underlineYellow", "58;5;3", 59, "Underline colour: yellow.";
                underline_blue => "underlineBlue", "58;5;4", 59, "Underline colour: blue.";
                underline_magenta => "underlineMagenta", "58;5;5", 59, "Underline colour: magenta.";
                underline_cyan => "underlineCyan", "58;5;6", 59, "Underline colour: cyan.";
                underline_white => "underlineWhite", "58;5;7", 59, "Underline colour: white.";

                underline_black_bright => "underlineBlackBright", "58;5;8", 59, "Underline colour: bright black.";
                underline_gray => "underlineGray", "58;5;8", 59, "Underline colour: alias for `underline_black_bright`.";
                underline_grey => "underlineGrey", "58;5;8", 59, "Underline colour: alias for `underline_black_bright`.";
                underline_red_bright => "underlineRedBright", "58;5;9", 59, "Underline colour: bright red.";
                underline_green_bright => "underlineGreenBright", "58;5;10", 59, "Underline colour: bright green.";
                underline_yellow_bright => "underlineYellowBright", "58;5;11", 59, "Underline colour: bright yellow.";
                underline_blue_bright => "underlineBlueBright", "58;5;12", 59, "Underline colour: bright blue.";
                underline_magenta_bright => "underlineMagentaBright", "58;5;13", 59, "Underline colour: bright magenta.";
                underline_cyan_bright => "underlineCyanBright", "58;5;14", 59, "Underline colour: bright cyan.";
                underline_white_bright => "underlineWhiteBright", "58;5;15", 59, "Underline colour: bright white.";
            }
        }
    };
}

pub(crate) use with_style_table;

/// Generates one accessor per named style, plus the per-group tables.
macro_rules! define_ansi_styles {
    (
        modifier { $($m:ident => $mn:literal, $mo:literal, $mc:literal, $md:literal;)* }
        color { $($c:ident => $cn:literal, $co:literal, $cc:literal, $cd:literal;)* }
        bg_color { $($b:ident => $bn:literal, $bo:literal, $bc:literal, $bd:literal;)* }
        underline_color { $($u:ident => $un:literal, $uo:literal, $uc:literal, $ud:literal;)* }
    ) => {
        $(
            #[doc = $md]
            #[must_use]
            pub const fn $m() -> CsPair {
                CsPair { open: concat!("\u{1b}[", $mo, "m"), close: concat!("\u{1b}[", $mc, "m") }
            }
        )*
        $(
            #[doc = $cd]
            #[must_use]
            pub const fn $c() -> CsPair {
                CsPair { open: concat!("\u{1b}[", $co, "m"), close: concat!("\u{1b}[", $cc, "m") }
            }
        )*
        $(
            #[doc = $bd]
            #[must_use]
            pub const fn $b() -> CsPair {
                CsPair { open: concat!("\u{1b}[", $bo, "m"), close: concat!("\u{1b}[", $bc, "m") }
            }
        )*
        $(
            #[doc = $ud]
            #[must_use]
            pub const fn $u() -> CsPair {
                CsPair { open: concat!("\u{1b}[", $uo, "m"), close: concat!("\u{1b}[", $uc, "m") }
            }
        )*

        /// Every modifier, in declaration order.
        pub const MODIFIER: &[(&str, CsPair)] = &[$(($mn, $m())),*];
        /// Every foreground colour, in declaration order.
        pub const COLOR: &[(&str, CsPair)] = &[$(($cn, $c())),*];
        /// Every background colour, in declaration order.
        pub const BG_COLOR: &[(&str, CsPair)] = &[$(($bn, $b())),*];
        /// Every underline colour, in declaration order.
        pub const UNDERLINE_COLOR: &[(&str, CsPair)] = &[$(($un, $u())),*];

        /// The modifier names, in declaration order.
        pub const MODIFIER_NAMES: &[&str] = &[$($mn),*];
        /// The foreground colour names, in declaration order.
        pub const FOREGROUND_COLOR_NAMES: &[&str] = &[$($cn),*];
        /// The background colour names, in declaration order.
        pub const BACKGROUND_COLOR_NAMES: &[&str] = &[$($bn),*];
        /// The underline colour names, in declaration order.
        pub const UNDERLINE_COLOR_NAMES: &[&str] = &[$($un),*];
    };
}

with_style_table!(define_ansi_styles);

/// The foreground colour names followed by the background colour names.
///
/// Underline colour names are deliberately **not** part of this list, matching
/// the original.
pub static COLOR_NAMES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let mut names = FOREGROUND_COLOR_NAMES.to_vec();
    names.extend_from_slice(BACKGROUND_COLOR_NAMES);
    names
});

/// Every named style, in group-declaration order.
fn all_styles() -> impl Iterator<Item = &'static (&'static str, CsPair)> {
    MODIFIER
        .iter()
        .chain(COLOR)
        .chain(BG_COLOR)
        .chain(UNDERLINE_COLOR)
}

/// Look a named style up by its name, as spelled in the exported name lists.
#[must_use]
pub fn style(name: &str) -> Option<CsPair> {
    all_styles()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, pair)| *pair)
}

/// A map from the leading SGR parameter of a style's open sequence to the SGR
/// parameter that closes it.
///
/// Only the leading parameter identifies a style, so `4:3` is keyed as `4` and
/// `58;5;1` is keyed as `58`. Entries are in first-insertion order.
pub static CODES: LazyLock<Vec<(u16, u16)>> = LazyLock::new(|| {
    let mut order: Vec<u16> = Vec::new();
    let mut values: HashMap<u16, u16> = HashMap::new();

    for (_, pair) in all_styles() {
        let open = leading_sgr_parameter(pair.open);
        let close = leading_sgr_parameter(pair.close);
        if values.insert(open, close).is_none() {
            order.push(open);
        }
    }

    order.into_iter().map(|key| (key, values[&key])).collect()
});

/// The leading base-10 integer of an SGR sequence's parameter list.
///
/// Mirrors `Number.parseInt(parameter, 10)`, which stops at the first character
/// that is not a digit.
fn leading_sgr_parameter(sequence: &str) -> u16 {
    let digits: String = sequence
        .trim_start_matches("\u{1b}[")
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().unwrap_or(0)
}

/// `Math.round`, which rounds a tie towards positive infinity.
///
/// Every value this is applied to is non-negative and far inside `u8`, so
/// `f64::round` — which rounds a tie away from zero — is equivalent.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "every caller passes a non-negative value that is at most 255"
)]
fn js_round(value: f64) -> u8 {
    value.round() as u8
}

/// Convert from the RGB colour space to the ANSI 256 colour space.
///
/// From <https://github.com/Qix-/color-convert/blob/3f0e0d4e92e235796ccb17f6e85c72094a651f49/conversions.js>
#[must_use]
pub fn rgb_to_ansi256(red: u8, green: u8, blue: u8) -> u8 {
    // We use the extended greyscale palette here, with the exception of
    // black and white. normal palette only has 4 greyscale shades.
    if red == green && green == blue {
        if red < 8 {
            return 16;
        }

        if red > 248 {
            return 231;
        }

        return js_round(f64::from(red - 8) / 247.0 * 24.0) + 232;
    }

    16 + (36 * js_round(f64::from(red) / 255.0 * 5.0))
        + (6 * js_round(f64::from(green) / 255.0 * 5.0))
        + js_round(f64::from(blue) / 255.0 * 5.0)
}

/// Convert from the RGB HEX colour space to the RGB colour space.
///
/// Reproduces `/[\da-f]{6}|[\da-f]{3}/i` applied to the input: the leftmost
/// run of hex digits wins, and at each position six digits are preferred over
/// three. A three-digit match is expanded by doubling each digit. An input with
/// no match at all is black.
#[must_use]
pub fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let Some(matched) = first_hex_run(hex) else {
        return (0, 0, 0);
    };

    let expanded = if matched.len() == 3 {
        matched.chars().flat_map(|digit| [digit, digit]).collect()
    } else {
        matched.to_owned()
    };

    let integer = u32::from_str_radix(&expanded, 16).unwrap_or(0);

    (
        ((integer >> 16) & 0xFF) as u8,
        ((integer >> 8) & 0xFF) as u8,
        (integer & 0xFF) as u8,
    )
}

/// The leftmost run of six — failing that, three — hexadecimal digits.
fn first_hex_run(hex: &str) -> Option<&str> {
    let bytes = hex.as_bytes();

    for start in 0..bytes.len() {
        let run = bytes[start..]
            .iter()
            .take_while(|byte| byte.is_ascii_hexdigit())
            .count();

        if run >= 6 {
            return Some(&hex[start..start + 6]);
        }

        if run >= 3 {
            return Some(&hex[start..start + 3]);
        }
    }

    None
}

/// Convert from the RGB HEX colour space to the ANSI 256 colour space.
#[must_use]
pub fn hex_to_ansi256(hex: &str) -> u8 {
    let (red, green, blue) = hex_to_rgb(hex);
    rgb_to_ansi256(red, green, blue)
}

/// Convert from the ANSI 256 colour space to the ANSI 16 colour space.
#[expect(
    clippy::float_cmp,
    reason = "the original compares against 0 and 2 exactly; an epsilon would \
              change which colours are considered fully saturated"
)]
#[must_use]
pub fn ansi256_to_ansi(code: u8) -> u8 {
    if code < 8 {
        return 30 + code;
    }

    if code < 16 {
        return 90 + (code - 8);
    }

    let (red, green, blue) = if code >= 232 {
        let shade = ((f64::from(code) - 232.0) * 10.0 + 8.0) / 255.0;
        (shade, shade, shade)
    } else {
        let offset = u16::from(code) - 16;
        let remainder = offset % 36;

        (
            f64::from(offset / 36) / 5.0,
            f64::from(remainder / 6) / 5.0,
            f64::from(remainder % 6) / 5.0,
        )
    };

    let value = red.max(green).max(blue) * 2.0;

    if value == 0.0 {
        return 30;
    }

    let mut result = 30 + ((js_round(blue) << 2) | (js_round(green) << 1) | js_round(red));

    if value == 2.0 {
        result += 60;
    }

    result
}

/// Convert from the RGB colour space to the ANSI 16 colour space.
#[must_use]
pub fn rgb_to_ansi(red: u8, green: u8, blue: u8) -> u8 {
    ansi256_to_ansi(rgb_to_ansi256(red, green, blue))
}

/// Convert from the RGB HEX colour space to the ANSI 16 colour space.
#[must_use]
pub fn hex_to_ansi(hex: &str) -> u8 {
    ansi256_to_ansi(hex_to_ansi256(hex))
}
