//! Terminal string styling done right.
//!
//! ```
//! use chalk::chalk;
//!
//! println!("{}", chalk().blue().paint("Hello world!"));
//! ```
//!
//! Chalk comes with an easy to use composable API where you just chain and nest
//! the styles you want.
//!
//! ```
//! use chalk::chalk;
//!
//! let chalk = chalk();
//!
//! // Combine styled and normal strings
//! println!("{} World{}", chalk.blue().paint("Hello"), chalk.red().paint("!"));
//!
//! // Compose multiple styles using the chainable API
//! println!("{}", chalk.blue().bg_red().bold().paint("Hello world!"));
//!
//! // Pass in multiple arguments
//! println!("{}", chalk.blue().paint_all(["Hello", "World!", "Foo", "bar"]));
//!
//! // Nest styles
//! println!("{}", chalk.red().paint_all(["Hello", &format!("{}!", chalk.underline().bg_blue().paint("world"))]));
//!
//! // Nest styles of the same type even (color, underline, background)
//! println!("{}", chalk.green().paint(format!(
//!     "I am a green line {} that becomes green again!",
//!     chalk.blue().underline().bold().paint("with a blue substring"),
//! )));
//! ```
//!
//! Easily define your own themes:
//!
//! ```
//! use chalk::chalk;
//!
//! let chalk = chalk();
//! let error = chalk.bold().red();
//! let warning = chalk.hex("#FFA500");
//!
//! println!("{}", error.paint("Error!"));
//! println!("{}", warning.paint("Warning!"));
//! ```
//!
//! # Colour support
//!
//! Chalk auto-detects how much colour the terminal supports and silently emits
//! plain text when it supports none. Override the detection by constructing an
//! instance with an explicit [`ColorSupportLevel`], or by assigning one:
//!
//! ```
//! use chalk::{Chalk, ColorSupportLevel};
//!
//! let chalk = Chalk::with_level(ColorSupportLevel::TrueColor);
//! assert_eq!(chalk.red().paint("foo"), "\u{1b}[31mfoo\u{1b}[39m");
//!
//! chalk.set_level(ColorSupportLevel::None);
//! assert_eq!(chalk.red().paint("foo"), "foo");
//! ```

#![warn(missing_docs)]

use std::borrow::Cow;
use std::fmt::{self, Display};
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU8, Ordering};

pub mod utilities;
pub mod vendor;

use utilities::{string_encase_crlf_with_first_index, string_replace_all};
use vendor::ansi_styles::{self, ColorType, CsPair, with_style_table};
use vendor::supports_color::{ColorInfo, SUPPORTS_COLOR};

pub use vendor::ansi_styles::{
    BACKGROUND_COLOR_NAMES, COLOR_NAMES, FOREGROUND_COLOR_NAMES, MODIFIER_NAMES,
    UNDERLINE_COLOR_NAMES,
};
pub use vendor::supports_color::{ColorSupport, SupportsColor};

/// The error produced by a level that is not an integer from 0 to 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidLevel;

impl Display for InvalidLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("The `level` should be an integer from 0 to 3")
    }
}

impl std::error::Error for InvalidLevel {}

/// How much colour Chalk should emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ColorSupportLevel {
    /// All colours disabled.
    #[default]
    None = 0,
    /// Basic 16 colours support.
    Basic = 1,
    /// ANSI 256 colours support.
    Ansi256 = 2,
    /// Truecolor 16 million colours support.
    TrueColor = 3,
}

impl ColorSupportLevel {
    /// The level as the number the original exposes.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// A level that detection produced, which is always in range.
    pub(crate) const fn from_detected(level: u8) -> Self {
        match level {
            0 => Self::None,
            1 => Self::Basic,
            2 => Self::Ansi256,
            _ => Self::TrueColor,
        }
    }
}

impl TryFrom<i64> for ColorSupportLevel {
    type Error = InvalidLevel;

    fn try_from(level: i64) -> Result<Self, Self::Error> {
        match level {
            0 => Ok(Self::None),
            1 => Ok(Self::Basic),
            2 => Ok(Self::Ansi256),
            3 => Ok(Self::TrueColor),
            _ => Err(InvalidLevel),
        }
    }
}

impl TryFrom<f64> for ColorSupportLevel {
    type Error = InvalidLevel;

    /// Rejects anything that is not a whole number, mirroring the original's
    /// `Number.isSafeInteger` check.
    fn try_from(level: f64) -> Result<Self, Self::Error> {
        if !level.is_finite() || level.fract() != 0.0 {
            return Err(InvalidLevel);
        }

        #[expect(
            clippy::cast_possible_truncation,
            reason = "the value is finite and has no fractional part"
        )]
        Self::try_from(level as i64)
    }
}

impl std::str::FromStr for ColorSupportLevel {
    type Err = InvalidLevel;

    /// Rejects surrounding whitespace and any other non-integer spelling, the
    /// way the original rejects the string `" 1"`.
    fn from_str(level: &str) -> Result<Self, Self::Err> {
        Self::try_from(level.parse::<i64>().map_err(|_| InvalidLevel)?)
    }
}

impl Display for ColorSupportLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.as_u8())
    }
}

/// Options for constructing a [`Chalk`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Options {
    /// Specify the colour support for Chalk.
    ///
    /// `None` — the default — has the level detected from the environment
    /// instead.
    pub level: Option<ColorSupportLevel>,
}

/// One link of a style chain, and the accumulated sequences up to it.
#[derive(Debug)]
struct Styler {
    /// This link's own opening sequence.
    open: Cow<'static, str>,
    /// This link's own closing sequence.
    close: Cow<'static, str>,
    /// Every opening sequence from the outermost link inwards.
    open_all: String,
    /// Every closing sequence from this link outwards.
    close_all: String,
    /// The link this one was chained onto.
    parent: Option<Arc<Styler>>,
}

impl Styler {
    /// Chain a style pair onto an optional parent link.
    fn new(open: Cow<'static, str>, close: Cow<'static, str>, parent: Option<Arc<Self>>) -> Self {
        let (open_all, close_all) = match &parent {
            None => (open.to_string(), close.to_string()),
            Some(parent) => (
                format!("{}{open}", parent.open_all),
                format!("{close}{}", parent.close_all),
            ),
        };

        Self {
            open,
            close,
            open_all,
            close_all,
            parent,
        }
    }
}

/// A style chain, ready to be applied to a value.
///
/// A style is obtained from a [`Chalk`] instance, or from another style, and
/// reads its level from the instance it originated on — however deep the chain
/// — so that changing that instance's level changes what an already-obtained
/// chain emits.
#[derive(Debug, Clone)]
pub struct Style {
    /// The level cell of the instance this chain started on.
    generator: Arc<AtomicU8>,
    /// The innermost link of the chain, if any.
    styler: Option<Arc<Styler>>,
    /// Whether the chain renders as the empty string when colour is off.
    is_empty: bool,
}

/// A Chalk instance: an isolated colour level plus the styles that read it.
///
/// Cloning an instance produces another handle to the *same* level, the way
/// assigning the object does in the original. Use [`Chalk::new`] or
/// [`Chalk::with_level`] for an independent one.
#[derive(Debug, Clone)]
pub struct Chalk {
    /// This instance's level, shared with every style obtained from it.
    level: Arc<AtomicU8>,
}

impl Default for Chalk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chalk {
    /// A new instance whose level is detected from the environment.
    #[must_use]
    pub fn new() -> Self {
        Self::with_options(Options::default())
    }

    /// A new instance configured by `options`.
    #[must_use]
    pub fn with_options(options: Options) -> Self {
        // Detect the level if it was not set manually.
        let level = options.level.unwrap_or_else(|| {
            SUPPORTS_COLOR
                .stdout
                .map_or(ColorSupportLevel::None, |support| support.level)
        });

        Self {
            level: Arc::new(AtomicU8::new(level.as_u8())),
        }
    }

    /// A new instance with an explicit level.
    #[must_use]
    pub fn with_level(level: ColorSupportLevel) -> Self {
        Self::with_options(Options { level: Some(level) })
    }

    /// A new instance with a level that still has to be validated.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidLevel`] if the value is not an integer from 0 to 3.
    pub fn try_with_level<T>(level: T) -> Result<Self, InvalidLevel>
    where
        ColorSupportLevel: TryFrom<T, Error = InvalidLevel>,
    {
        Ok(Self::with_level(ColorSupportLevel::try_from(level)?))
    }

    /// The colour support for this instance.
    #[must_use]
    pub fn level(&self) -> ColorSupportLevel {
        ColorSupportLevel::from_detected(self.level.load(Ordering::Relaxed))
    }

    /// Set the colour support for this instance.
    ///
    /// This takes `&self` rather than `&mut self` because every style obtained
    /// from an instance shares that instance's level: assigning through a
    /// chain has to reach the instance it started on.
    pub fn set_level(&self, level: ColorSupportLevel) {
        self.level.store(level.as_u8(), Ordering::Relaxed);
    }

    /// Set the colour support from a value that still has to be validated.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidLevel`] if the value is not an integer from 0 to 3.
    pub fn try_set_level<T>(&self, level: T) -> Result<(), InvalidLevel>
    where
        ColorSupportLevel: TryFrom<T, Error = InvalidLevel>,
    {
        self.set_level(ColorSupportLevel::try_from(level)?);
        Ok(())
    }

    /// The unstyled chain rooted at this instance.
    fn root(&self) -> Style {
        Style {
            generator: Arc::clone(&self.level),
            styler: None,
            is_empty: false,
        }
    }

    /// Stringify a value, applying no styling at all.
    ///
    /// The instance itself is not a style: calling it only converts, exactly as
    /// calling the original's exported object does.
    #[must_use]
    pub fn paint<T: Display>(&self, value: T) -> String {
        value.to_string()
    }

    /// Stringify several values and join them with a space, applying no styling.
    #[must_use]
    pub fn paint_all<I>(&self, values: I) -> String
    where
        I: IntoIterator,
        I::Item: Display,
    {
        join_with_space(values)
    }
}

impl Style {
    /// The colour support of the instance this chain started on.
    #[must_use]
    pub fn level(&self) -> ColorSupportLevel {
        ColorSupportLevel::from_detected(self.generator.load(Ordering::Relaxed))
    }

    /// Set the colour support of the instance this chain started on.
    pub fn set_level(&self, level: ColorSupportLevel) {
        self.generator.store(level.as_u8(), Ordering::Relaxed);
    }

    /// Set that level from a value that still has to be validated.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidLevel`] if the value is not an integer from 0 to 3.
    pub fn try_set_level<T>(&self, level: T) -> Result<(), InvalidLevel>
    where
        ColorSupportLevel: TryFrom<T, Error = InvalidLevel>,
    {
        self.set_level(ColorSupportLevel::try_from(level)?);
        Ok(())
    }

    /// Apply this chain to a value.
    #[must_use]
    pub fn paint<T: Display>(&self, value: T) -> String {
        self.apply_style(value.to_string())
    }

    /// Apply this chain to several values, joined with a space.
    #[must_use]
    pub fn paint_all<I>(&self, values: I) -> String
    where
        I: IntoIterator,
        I::Item: Display,
    {
        self.apply_style(join_with_space(values))
    }

    /// Wrap `string` in this chain's escape sequences.
    fn apply_style(&self, string: String) -> String {
        if self.level() == ColorSupportLevel::None || string.is_empty() {
            return if self.is_empty { String::new() } else { string };
        }

        let Some(styler) = &self.styler else {
            return string;
        };

        let mut string = string;

        if string.contains('\u{1b}') {
            let mut link = Some(styler);

            while let Some(current) = link {
                // Replace any instances already present with a re-opening code
                // otherwise only the part of the string until said closing code
                // will be colored, and the rest will simply be 'plain'.
                string = string_replace_all(&string, &current.close, &current.open).into_owned();

                link = current.parent.as_ref();
            }
        }

        // We can move both next actions out of loop, because remaining actions in loop won't have
        // any/visible effect on parts we add here. Close the styling before a linebreak and reopen
        // after next line to fix a bleed issue on macOS: https://github.com/chalk/chalk/pull/92
        if let Some(lf_index) = string.find('\n') {
            string = string_encase_crlf_with_first_index(
                &string,
                &styler.close_all,
                &styler.open_all,
                lf_index,
            );
        }

        let mut wrapped =
            String::with_capacity(styler.open_all.len() + string.len() + styler.close_all.len());
        wrapped.push_str(&styler.open_all);
        wrapped.push_str(&string);
        wrapped.push_str(&styler.close_all);
        wrapped
    }
}

/// Stringify several values and join them with a single space.
fn join_with_space<I>(values: I) -> String
where
    I: IntoIterator,
    I::Item: Display,
{
    let mut joined = String::new();

    for (index, value) in values.into_iter().enumerate() {
        if index > 0 {
            joined.push(' ');
        }

        joined.push_str(&value.to_string());
    }

    joined
}

/// Everything a style chain can be extended by.
///
/// Implemented for [`Chalk`] and [`Style`] alike so that a chain reads the same
/// whether it starts on an instance or continues from another style.
trait Chain {
    /// The chain this style would extend.
    fn as_style(&self) -> Style;

    /// Extend the chain with one named style.
    fn chain(&self, pair: CsPair) -> Style {
        self.chain_owned(Cow::Borrowed(pair.open), Cow::Borrowed(pair.close))
    }

    /// Extend the chain with a dynamically built sequence pair.
    fn chain_owned(&self, open: Cow<'static, str>, close: Cow<'static, str>) -> Style {
        let base = self.as_style();

        Style {
            styler: Some(Arc::new(Styler::new(open, close, base.styler))),
            generator: base.generator,
            is_empty: base.is_empty,
        }
    }

    /// Mark the chain as rendering to nothing when colour is off.
    fn chain_visible(&self) -> Style {
        Style {
            is_empty: true,
            ..self.as_style()
        }
    }
}

impl Chain for Chalk {
    fn as_style(&self) -> Style {
        self.root()
    }
}

impl Chain for Style {
    fn as_style(&self) -> Style {
        self.clone()
    }
}

/// Build the open sequence for a colour model at a given level.
///
/// Resolving the model and the level together in one place keeps the three
/// model families from each re-deciding what a level means. The level-0 arm is
/// present, and evaluated, but its result is discarded: `apply_style`
/// short-circuits before it can be used.
fn model_rgb(kind: ColorType, level: ColorSupportLevel, red: u8, green: u8, blue: u8) -> String {
    match level {
        ColorSupportLevel::None | ColorSupportLevel::Basic => {
            kind.ansi(ansi_styles::rgb_to_ansi(red, green, blue))
        }
        ColorSupportLevel::Ansi256 => kind.ansi256(ansi_styles::rgb_to_ansi256(red, green, blue)),
        ColorSupportLevel::TrueColor => kind.ansi16m(red, green, blue),
    }
}

/// Build the open sequence for a hex colour at a given level.
fn model_hex(kind: ColorType, level: ColorSupportLevel, hex: &str) -> String {
    match level {
        ColorSupportLevel::None | ColorSupportLevel::Basic => {
            kind.ansi(ansi_styles::hex_to_ansi(hex))
        }
        ColorSupportLevel::Ansi256 => kind.ansi256(ansi_styles::hex_to_ansi256(hex)),
        ColorSupportLevel::TrueColor => {
            let (red, green, blue) = ansi_styles::hex_to_rgb(hex);
            kind.ansi16m(red, green, blue)
        }
    }
}

/// Build the open sequence for a palette index at a given level.
///
/// `ansi256` is already the native form, so only the 16-colour levels need
/// converting.
fn model_ansi256(kind: ColorType, level: ColorSupportLevel, code: u8) -> String {
    match level {
        ColorSupportLevel::None | ColorSupportLevel::Basic => {
            kind.ansi(ansi_styles::ansi256_to_ansi(code))
        }
        ColorSupportLevel::Ansi256 | ColorSupportLevel::TrueColor => kind.ansi256(code),
    }
}

/// Generates the named-style accessors for one type.
macro_rules! define_style_methods {
    (
        modifier { $($m:ident => $mn:literal, $mo:literal, $mc:literal, $md:literal;)* }
        color { $($c:ident => $cn:literal, $co:literal, $cc:literal, $cd:literal;)* }
        bg_color { $($b:ident => $bn:literal, $bo:literal, $bc:literal, $bd:literal;)* }
        underline_color { $($u:ident => $un:literal, $uo:literal, $uc:literal, $ud:literal;)* }
    ) => {
        $(
            #[doc = $md]
            #[must_use]
            pub fn $m(&self) -> Style { self.chain(ansi_styles::$m()) }
        )*
        $(
            #[doc = $cd]
            #[must_use]
            pub fn $c(&self) -> Style { self.chain(ansi_styles::$c()) }
        )*
        $(
            #[doc = $bd]
            #[must_use]
            pub fn $b(&self) -> Style { self.chain(ansi_styles::$b()) }
        )*
        $(
            #[doc = $ud]
            #[must_use]
            pub fn $u(&self) -> Style { self.chain(ansi_styles::$u()) }
        )*

        /// Look a style up by the name it has in the exported name lists.
        ///
        /// The names are the original's, so they are camel case — they are data
        /// that callers match on, not Rust identifiers.
        #[must_use]
        pub fn style_by_name(&self, name: &str) -> Option<Style> {
            ansi_styles::style(name).map(|pair| self.chain(pair))
        }
    };
}

/// Generates the model-colour and `visible` accessors for one type.
macro_rules! define_model_methods {
    () => {
        /// Use RGB values to set text colour.
        #[must_use]
        pub fn rgb(&self, red: u8, green: u8, blue: u8) -> Style {
            let open = model_rgb(
                ColorType::Foreground,
                self.as_style().level(),
                red,
                green,
                blue,
            );
            self.chain_owned(
                Cow::Owned(open),
                Cow::Borrowed(ColorType::Foreground.close()),
            )
        }

        /// Use a HEX value to set text colour.
        #[must_use]
        pub fn hex(&self, color: &str) -> Style {
            let open = model_hex(ColorType::Foreground, self.as_style().level(), color);
            self.chain_owned(
                Cow::Owned(open),
                Cow::Borrowed(ColorType::Foreground.close()),
            )
        }

        /// Use an 8-bit palette index to set text colour.
        ///
        /// The value is downsampled to the 16-colour palette on terminals that
        /// only support basic colours, so `ansi256(196)` becomes `SGR 91`.
        #[must_use]
        pub fn ansi256(&self, index: u8) -> Style {
            let open = model_ansi256(ColorType::Foreground, self.as_style().level(), index);
            self.chain_owned(
                Cow::Owned(open),
                Cow::Borrowed(ColorType::Foreground.close()),
            )
        }

        /// Use RGB values to set background colour.
        #[must_use]
        pub fn bg_rgb(&self, red: u8, green: u8, blue: u8) -> Style {
            let open = model_rgb(
                ColorType::Background,
                self.as_style().level(),
                red,
                green,
                blue,
            );
            self.chain_owned(
                Cow::Owned(open),
                Cow::Borrowed(ColorType::Background.close()),
            )
        }

        /// Use a HEX value to set background colour.
        #[must_use]
        pub fn bg_hex(&self, color: &str) -> Style {
            let open = model_hex(ColorType::Background, self.as_style().level(), color);
            self.chain_owned(
                Cow::Owned(open),
                Cow::Borrowed(ColorType::Background.close()),
            )
        }

        /// Use an 8-bit palette index to set background colour.
        #[must_use]
        pub fn bg_ansi256(&self, index: u8) -> Style {
            let open = model_ansi256(ColorType::Background, self.as_style().level(), index);
            self.chain_owned(
                Cow::Owned(open),
                Cow::Borrowed(ColorType::Background.close()),
            )
        }

        /// Use RGB values to set underline colour.
        ///
        /// The underline colour is only visible when an underline style is also
        /// applied.
        #[must_use]
        pub fn underline_rgb(&self, red: u8, green: u8, blue: u8) -> Style {
            let open = model_rgb(
                ColorType::Underline,
                self.as_style().level(),
                red,
                green,
                blue,
            );
            self.chain_owned(
                Cow::Owned(open),
                Cow::Borrowed(ColorType::Underline.close()),
            )
        }

        /// Use a HEX value to set underline colour.
        ///
        /// The underline colour is only visible when an underline style is also
        /// applied.
        #[must_use]
        pub fn underline_hex(&self, color: &str) -> Style {
            let open = model_hex(ColorType::Underline, self.as_style().level(), color);
            self.chain_owned(
                Cow::Owned(open),
                Cow::Borrowed(ColorType::Underline.close()),
            )
        }

        /// Use an 8-bit palette index to set underline colour.
        ///
        /// The value is downsampled to the first 16 palette entries on
        /// terminals that only support basic colours, so `underline_ansi256(196)`
        /// becomes palette index 9.
        #[must_use]
        pub fn underline_ansi256(&self, index: u8) -> Style {
            let open = model_ansi256(ColorType::Underline, self.as_style().level(), index);
            self.chain_owned(
                Cow::Owned(open),
                Cow::Borrowed(ColorType::Underline.close()),
            )
        }

        /// Print the text only when Chalk has a colour level above zero.
        ///
        /// Can be useful for things that are purely cosmetic.
        #[must_use]
        pub fn visible(&self) -> Style {
            self.chain_visible()
        }
    };
}

impl Chalk {
    with_style_table!(define_style_methods);
    define_model_methods!();
}

impl Style {
    with_style_table!(define_style_methods);
    define_model_methods!();
}

/// The default instance, whose level is detected from standard output.
///
/// ```
/// use chalk::chalk;
///
/// println!("{}", chalk().green().paint("Hello!"));
/// ```
#[must_use]
pub fn chalk() -> &'static Chalk {
    static CHALK: LazyLock<Chalk> = LazyLock::new(Chalk::new);
    &CHALK
}

/// An instance whose level is detected from standard error.
#[must_use]
pub fn chalk_stderr() -> &'static Chalk {
    static CHALK_STDERR: LazyLock<Chalk> = LazyLock::new(|| {
        Chalk::with_level(
            SUPPORTS_COLOR
                .stderr
                .map_or(ColorSupportLevel::None, |support| support.level),
        )
    });
    &CHALK_STDERR
}

/// The colour support detected for standard output.
#[must_use]
pub fn supports_color() -> ColorInfo {
    SUPPORTS_COLOR.stdout
}

/// The colour support detected for standard error.
#[must_use]
pub fn supports_color_stderr() -> ColorInfo {
    SUPPORTS_COLOR.stderr
}
