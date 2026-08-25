//! Port of `test/chalk.js`.

use std::fmt;
use std::sync::Once;

use chalk::{
    COLOR_NAMES, Chalk, ColorSupportLevel, MODIFIER_NAMES, UNDERLINE_COLOR_NAMES, chalk,
    chalk_stderr,
};

/// The original sets both shared instances to level 3 at the top of the file
/// and prints two diagnostics. Every test here only ever reads that level, so
/// running the preamble once per test is enough — no test can observe another
/// having run it.
fn setup() {
    static SETUP: Once = Once::new();

    SETUP.call_once(|| {
        chalk().set_level(ColorSupportLevel::TrueColor);
        chalk_stderr().set_level(ColorSupportLevel::TrueColor);

        println!(
            "TERM: {}",
            std::env::var("TERM").unwrap_or_else(|_| "[none]".into())
        );
        println!("platform: {}", std::env::consts::OS);
    });
}

/// A value whose string conversion is not simply its own text, standing in for
/// the original's `['hello', 'there']`. JavaScript coerces an array by joining
/// it with commas; Rust converts a value through its own `Display`.
struct List(&'static [&'static str]);

impl fmt::Display for List {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0.join(","))
    }
}

/// `don't add any styling when called as the base function`
#[test]
fn dont_add_any_styling_when_called_as_the_base_function() {
    setup();
    assert_eq!(chalk().paint("foo"), "foo");
}

/// `support multiple arguments in base function`
#[test]
fn support_multiple_arguments_in_base_function() {
    setup();
    assert_eq!(chalk().paint_all(["hello", "there"]), "hello there");
}

/// `support automatic casting to string`
#[test]
fn support_automatic_casting_to_string() {
    setup();
    assert_eq!(chalk().paint(List(&["hello", "there"])), "hello,there");
    assert_eq!(chalk().paint(123), "123");

    assert_eq!(
        chalk().bold().paint(List(&["foo", "bar"])),
        "\u{1b}[1mfoo,bar\u{1b}[22m"
    );
    assert_eq!(chalk().green().paint(98_765), "\u{1b}[32m98765\u{1b}[39m");
}

/// `style string`
#[test]
fn style_string() {
    setup();
    assert_eq!(chalk().underline().paint("foo"), "\u{1b}[4mfoo\u{1b}[24m");
    assert_eq!(chalk().red().paint("foo"), "\u{1b}[31mfoo\u{1b}[39m");
    assert_eq!(chalk().bg_red().paint("foo"), "\u{1b}[41mfoo\u{1b}[49m");
}

/// `support applying multiple styles at once`
#[test]
fn support_applying_multiple_styles_at_once() {
    setup();
    assert_eq!(
        chalk().red().bg_green().underline().paint("foo"),
        "\u{1b}[31m\u{1b}[42m\u{1b}[4mfoo\u{1b}[24m\u{1b}[49m\u{1b}[39m"
    );
    assert_eq!(
        chalk().underline().red().bg_green().paint("foo"),
        "\u{1b}[4m\u{1b}[31m\u{1b}[42mfoo\u{1b}[49m\u{1b}[39m\u{1b}[24m"
    );
}

/// `support nesting styles`
#[test]
fn support_nesting_styles() {
    setup();
    assert_eq!(
        chalk().red().paint(format!(
            "foo{}!",
            chalk().underline().bg_blue().paint("bar")
        )),
        "\u{1b}[31mfoo\u{1b}[4m\u{1b}[44mbar\u{1b}[49m\u{1b}[24m!\u{1b}[39m"
    );
}

/// `support nesting styles of the same type (color, underline, bg)`
#[test]
fn support_nesting_styles_of_the_same_type() {
    setup();
    let nested = chalk()
        .yellow()
        .paint(format!("b{}b", chalk().green().paint("c")));

    assert_eq!(
        chalk().red().paint(format!("a{nested}c")),
        "\u{1b}[31ma\u{1b}[33mb\u{1b}[32mc\u{1b}[39m\u{1b}[31m\u{1b}[33mb\u{1b}[39m\u{1b}[31mc\u{1b}[39m"
    );
}

/// ``reset all styles with `.reset()` ``
#[test]
fn reset_all_styles_with_reset() {
    setup();
    assert_eq!(
        chalk().reset().paint(format!(
            "{}foo",
            chalk().red().bg_green().underline().paint("foo")
        )),
        "\u{1b}[0m\u{1b}[31m\u{1b}[42m\u{1b}[4mfoo\u{1b}[24m\u{1b}[49m\u{1b}[39mfoo\u{1b}[0m"
    );
}

/// `support caching multiple styles`
#[test]
fn support_caching_multiple_styles() {
    setup();
    let red = chalk().red().red();
    let green = chalk().red().green();
    let red_bold = red.bold();
    let green_bold = green.bold();

    assert_ne!(red.paint("foo"), green.paint("foo"));
    assert_ne!(red_bold.paint("bar"), green_bold.paint("bar"));
    assert_ne!(green.paint("baz"), green_bold.paint("baz"));
}

/// `alias gray to grey`
#[test]
fn alias_gray_to_grey() {
    setup();
    assert_eq!(chalk().grey().paint("foo"), "\u{1b}[90mfoo\u{1b}[39m");
}

/// `support variable number of arguments`
#[test]
fn support_variable_number_of_arguments() {
    setup();
    assert_eq!(
        chalk().red().paint_all(["foo", "bar"]),
        "\u{1b}[31mfoo bar\u{1b}[39m"
    );
}

/// `support falsy values`
#[test]
fn support_falsy_values() {
    setup();
    assert_eq!(chalk().red().paint(0), "\u{1b}[31m0\u{1b}[39m");
}

/// `don't output escape codes if the input is empty`
#[test]
fn dont_output_escape_codes_if_the_input_is_empty() {
    setup();
    assert_eq!(chalk().red().paint(""), "");
    assert_eq!(chalk().red().blue().black().paint(""), "");
}

/// `keep Function.prototype methods`
///
/// A builder is a first-class function in the original, so it survives being
/// detached from the object it came from and invoked indirectly. A [`Style`] is
/// a first-class value here, so the equivalent is detaching it into a binding
/// and applying it through an indirect call.
#[test]
fn keep_function_prototype_methods() {
    setup();
    let detached = chalk().grey();
    let apply = |style: &chalk::Style, text: &str| style.paint(text);
    assert_eq!(apply(&detached, "foo"), "\u{1b}[90mfoo\u{1b}[39m");

    let bound = chalk().red().bg_green().underline();
    assert_eq!(
        chalk().reset().paint(format!("{}foo", bound.paint("foo"))),
        "\u{1b}[0m\u{1b}[31m\u{1b}[42m\u{1b}[4mfoo\u{1b}[24m\u{1b}[49m\u{1b}[39mfoo\u{1b}[0m"
    );

    let empty = chalk().red().blue().black();
    assert_eq!(apply(&empty, ""), "");
}

/// `line breaks should open and close colors`
#[test]
fn line_breaks_should_open_and_close_colors() {
    setup();
    assert_eq!(
        chalk().grey().paint("hello\nworld"),
        "\u{1b}[90mhello\u{1b}[39m\n\u{1b}[90mworld\u{1b}[39m"
    );
}

/// `line breaks should open and close colors with CRLF`
#[test]
fn line_breaks_should_open_and_close_colors_with_crlf() {
    setup();
    assert_eq!(
        chalk().grey().paint("hello\r\nworld"),
        "\u{1b}[90mhello\u{1b}[39m\r\n\u{1b}[90mworld\u{1b}[39m"
    );
}

/// `properly convert RGB to 16 colors on basic color terminals`
#[test]
fn properly_convert_rgb_to_16_colors_on_basic_color_terminals() {
    setup();
    let basic = || Chalk::with_level(ColorSupportLevel::Basic);
    assert_eq!(
        basic().rgb(255, 0, 0).paint("hello"),
        "\u{1b}[91mhello\u{1b}[39m"
    );
    assert_eq!(
        basic().bg_rgb(255, 0, 0).paint("hello"),
        "\u{1b}[101mhello\u{1b}[49m"
    );
    assert_eq!(
        basic().hex("#FF0000").paint("hello"),
        "\u{1b}[91mhello\u{1b}[39m"
    );
    assert_eq!(
        basic().bg_hex("#FF0000").paint("hello"),
        "\u{1b}[101mhello\u{1b}[49m"
    );
}

/// `properly convert RGB to 256 colors on basic color terminals`
#[test]
fn properly_convert_rgb_to_256_colors_on_basic_color_terminals() {
    setup();
    let ansi256 = || Chalk::with_level(ColorSupportLevel::Ansi256);
    let truecolor = || Chalk::with_level(ColorSupportLevel::TrueColor);

    assert_eq!(
        ansi256().rgb(255, 0, 0).paint("hello"),
        "\u{1b}[38;5;196mhello\u{1b}[39m"
    );
    assert_eq!(
        ansi256().bg_rgb(255, 0, 0).paint("hello"),
        "\u{1b}[48;5;196mhello\u{1b}[49m"
    );
    assert_eq!(
        truecolor().rgb(255, 0, 0).paint("hello"),
        "\u{1b}[38;2;255;0;0mhello\u{1b}[39m"
    );
    assert_eq!(
        truecolor().bg_rgb(255, 0, 0).paint("hello"),
        "\u{1b}[48;2;255;0;0mhello\u{1b}[49m"
    );
    assert_eq!(
        ansi256().hex("#FF0000").paint("hello"),
        "\u{1b}[38;5;196mhello\u{1b}[39m"
    );
    assert_eq!(
        ansi256().bg_hex("#FF0000").paint("hello"),
        "\u{1b}[48;5;196mhello\u{1b}[49m"
    );
    assert_eq!(
        truecolor().bg_hex("#FF0000").paint("hello"),
        "\u{1b}[48;2;255;0;0mhello\u{1b}[49m"
    );
}

/// `properly convert ANSI 256 to 16 colors on basic color terminals`
#[test]
fn properly_convert_ansi_256_to_16_colors_on_basic_color_terminals() {
    setup();
    let basic = || Chalk::with_level(ColorSupportLevel::Basic);

    assert_eq!(
        basic().ansi256(196).paint("hello"),
        "\u{1b}[91mhello\u{1b}[39m"
    );
    assert_eq!(
        basic().bg_ansi256(196).paint("hello"),
        "\u{1b}[101mhello\u{1b}[49m"
    );
    assert_eq!(
        basic().ansi256(2).paint("hello"),
        "\u{1b}[32mhello\u{1b}[39m"
    );
    assert_eq!(
        basic().bg_ansi256(2).paint("hello"),
        "\u{1b}[42mhello\u{1b}[49m"
    );
    assert_eq!(
        basic().ansi256(8).paint("hello"),
        "\u{1b}[90mhello\u{1b}[39m"
    );
    assert_eq!(
        basic().ansi256(232).paint("hello"),
        "\u{1b}[30mhello\u{1b}[39m"
    );
    assert_eq!(
        basic().ansi256(255).paint("hello"),
        "\u{1b}[37mhello\u{1b}[39m"
    );
}

/// `keep ANSI 256 colors on 256 color and Truecolor terminals`
#[test]
fn keep_ansi_256_colors_on_256_color_and_truecolor_terminals() {
    setup();
    let ansi256 = || Chalk::with_level(ColorSupportLevel::Ansi256);
    let truecolor = || Chalk::with_level(ColorSupportLevel::TrueColor);

    assert_eq!(
        ansi256().ansi256(196).paint("hello"),
        "\u{1b}[38;5;196mhello\u{1b}[39m"
    );
    assert_eq!(
        ansi256().bg_ansi256(196).paint("hello"),
        "\u{1b}[48;5;196mhello\u{1b}[49m"
    );
    assert_eq!(
        truecolor().ansi256(196).paint("hello"),
        "\u{1b}[38;5;196mhello\u{1b}[39m"
    );
    assert_eq!(
        truecolor().bg_ansi256(196).paint("hello"),
        "\u{1b}[48;5;196mhello\u{1b}[49m"
    );
}

/// `don't emit color codes if level is 0`
#[test]
fn dont_emit_color_codes_if_level_is_0() {
    setup();
    let off = || Chalk::with_level(ColorSupportLevel::None);

    assert_eq!(off().hex("#FF0000").paint("hello"), "hello");
    assert_eq!(off().bg_hex("#FF0000").paint("hello"), "hello");
    assert_eq!(off().ansi256(196).paint("hello"), "hello");
    assert_eq!(off().bg_ansi256(196).paint("hello"), "hello");
    assert_eq!(off().underline_hex("#FF0000").paint("hello"), "hello");
    assert_eq!(off().underline_ansi256(196).paint("hello"), "hello");
    assert_eq!(off().underline_red().paint("hello"), "hello");
    assert_eq!(off().underline_curly().paint("hello"), "hello");
}

/// `support extended underline styles`
#[test]
fn support_extended_underline_styles() {
    setup();
    assert_eq!(
        chalk().underline_double().paint("foo"),
        "\u{1b}[4:2mfoo\u{1b}[24m"
    );
    assert_eq!(
        chalk().underline_curly().paint("foo"),
        "\u{1b}[4:3mfoo\u{1b}[24m"
    );
    assert_eq!(
        chalk().underline_dotted().paint("foo"),
        "\u{1b}[4:4mfoo\u{1b}[24m"
    );
    assert_eq!(
        chalk().underline_dashed().paint("foo"),
        "\u{1b}[4:5mfoo\u{1b}[24m"
    );
}

/// `support nesting underline styles`
#[test]
fn support_nesting_underline_styles() {
    setup();
    assert_eq!(
        chalk()
            .underline()
            .paint(format!("{}b", chalk().underline_curly().paint("a"))),
        "\u{1b}[4m\u{1b}[4:3ma\u{1b}[24m\u{1b}[4mb\u{1b}[24m"
    );

    assert_eq!(
        chalk()
            .underline_curly()
            .paint(format!("{}b", chalk().underline().paint("a"))),
        "\u{1b}[4:3m\u{1b}[4ma\u{1b}[24m\u{1b}[4:3mb\u{1b}[24m"
    );
}

/// `support underline colors`
#[test]
fn support_underline_colors() {
    setup();
    assert_eq!(
        chalk().underline_red().paint("foo"),
        "\u{1b}[58;5;1mfoo\u{1b}[59m"
    );
    assert_eq!(
        chalk().underline_black_bright().paint("foo"),
        "\u{1b}[58;5;8mfoo\u{1b}[59m"
    );
    assert_eq!(
        chalk().underline_gray().paint("foo"),
        chalk().underline_black_bright().paint("foo")
    );
    assert_eq!(
        chalk().underline_grey().paint("foo"),
        chalk().underline_black_bright().paint("foo")
    );
    assert_eq!(
        chalk().red().underline_red().underline_curly().paint("foo"),
        "\u{1b}[31m\u{1b}[58;5;1m\u{1b}[4:3mfoo\u{1b}[24m\u{1b}[59m\u{1b}[39m"
    );
}

/// `support nesting underline colors`
#[test]
fn support_nesting_underline_colors() {
    setup();
    assert_eq!(
        chalk()
            .underline_blue()
            .paint(format!("{}b", chalk().underline_red().paint("a"))),
        "\u{1b}[58;5;4m\u{1b}[58;5;1ma\u{1b}[59m\u{1b}[58;5;4mb\u{1b}[59m"
    );
}

/// `properly downsample underline colors`
#[test]
fn properly_downsample_underline_colors() {
    setup();
    let basic = || Chalk::with_level(ColorSupportLevel::Basic);
    let ansi256 = || Chalk::with_level(ColorSupportLevel::Ansi256);
    let truecolor = || Chalk::with_level(ColorSupportLevel::TrueColor);

    assert_eq!(
        truecolor().underline_rgb(255, 0, 0).paint("hello"),
        "\u{1b}[58;2;255;0;0mhello\u{1b}[59m"
    );
    assert_eq!(
        ansi256().underline_rgb(255, 0, 0).paint("hello"),
        "\u{1b}[58;5;196mhello\u{1b}[59m"
    );
    assert_eq!(
        basic().underline_rgb(255, 0, 0).paint("hello"),
        "\u{1b}[58;5;9mhello\u{1b}[59m"
    );
    assert_eq!(
        truecolor().underline_hex("#FF0000").paint("hello"),
        "\u{1b}[58;2;255;0;0mhello\u{1b}[59m"
    );
    assert_eq!(
        ansi256().underline_hex("#FF0000").paint("hello"),
        "\u{1b}[58;5;196mhello\u{1b}[59m"
    );
    assert_eq!(
        basic().underline_hex("#FF0000").paint("hello"),
        "\u{1b}[58;5;9mhello\u{1b}[59m"
    );
    assert_eq!(
        truecolor().underline_ansi256(196).paint("hello"),
        "\u{1b}[58;5;196mhello\u{1b}[59m"
    );
    assert_eq!(
        ansi256().underline_ansi256(196).paint("hello"),
        "\u{1b}[58;5;196mhello\u{1b}[59m"
    );
    assert_eq!(
        basic().underline_ansi256(196).paint("hello"),
        "\u{1b}[58;5;9mhello\u{1b}[59m"
    );
    assert_eq!(
        basic().underline_ansi256(2).paint("hello"),
        "\u{1b}[58;5;2mhello\u{1b}[59m"
    );
    assert_eq!(
        basic().underline_ansi256(232).paint("hello"),
        "\u{1b}[58;5;0mhello\u{1b}[59m"
    );

    // The named underline colors have no basic 16-color form, so they are the same at every level.
    assert_eq!(
        basic().underline_red().paint("hello"),
        "\u{1b}[58;5;1mhello\u{1b}[59m"
    );
    assert_eq!(
        ansi256().underline_red().paint("hello"),
        "\u{1b}[58;5;1mhello\u{1b}[59m"
    );
}

/// `expose the underline style names`
#[test]
fn expose_the_underline_style_names() {
    setup();
    assert!(MODIFIER_NAMES.contains(&"underlineCurly"));
    assert!(UNDERLINE_COLOR_NAMES.contains(&"underlineRedBright"));

    // Underline colors are intentionally not part of `colorNames`.
    assert!(!COLOR_NAMES.contains(&"underlineRed"));
}

/// `supports blackBright color`
#[test]
fn supports_black_bright_color() {
    setup();
    assert_eq!(
        chalk().black_bright().paint("foo"),
        "\u{1b}[90mfoo\u{1b}[39m"
    );
}

/// `sets correct level for chalkStderr and respects it`
#[test]
fn sets_correct_level_for_chalk_stderr_and_respects_it() {
    setup();
    assert_eq!(chalk_stderr().level(), ColorSupportLevel::TrueColor);
    assert_eq!(
        chalk_stderr().red().bold().paint("foo"),
        "\u{1b}[31m\u{1b}[1mfoo\u{1b}[22m\u{1b}[39m"
    );
}

/// `keeps function prototype methods`
///
/// The root object is callable in the original and keeps `Function.prototype`;
/// here it is a value that can be passed around and applied indirectly.
#[test]
fn keeps_function_prototype_methods() {
    setup();
    let apply = |instance: &Chalk, text: &str| instance.paint(text);

    assert_eq!(apply(chalk(), "foo"), "foo");
    assert_eq!(chalk().paint_all(["foo"]), "foo");
    assert_eq!(chalk().paint("foo"), "foo");
}
