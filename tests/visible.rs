//! Port of `test/visible.js`.

use chalk::{Chalk, ColorSupportLevel};

/// `visible: normal output when level > 0`
#[test]
fn visible_normal_output_when_level_above_0() {
    let instance = Chalk::with_level(ColorSupportLevel::TrueColor);
    assert_eq!(
        instance.visible().red().paint("foo"),
        "\u{1b}[31mfoo\u{1b}[39m"
    );
    assert_eq!(
        instance.red().visible().paint("foo"),
        "\u{1b}[31mfoo\u{1b}[39m"
    );
}

/// `visible: no output when level is too low`
#[test]
fn visible_no_output_when_level_is_too_low() {
    let instance = Chalk::with_level(ColorSupportLevel::None);
    assert_eq!(instance.visible().red().paint("foo"), "");
    assert_eq!(instance.red().visible().paint("foo"), "");
}

/// `test switching back and forth between level == 0 and level > 0`
#[test]
fn test_switching_back_and_forth_between_level_0_and_above() {
    let instance = Chalk::with_level(ColorSupportLevel::TrueColor);
    assert_eq!(instance.red().paint("foo"), "\u{1b}[31mfoo\u{1b}[39m");
    assert_eq!(
        instance.visible().red().paint("foo"),
        "\u{1b}[31mfoo\u{1b}[39m"
    );
    assert_eq!(
        instance.red().visible().paint("foo"),
        "\u{1b}[31mfoo\u{1b}[39m"
    );
    assert_eq!(instance.visible().paint("foo"), "foo");
    assert_eq!(instance.red().paint("foo"), "\u{1b}[31mfoo\u{1b}[39m");

    instance.set_level(ColorSupportLevel::None);
    assert_eq!(instance.red().paint("foo"), "foo");
    assert_eq!(instance.visible().paint("foo"), "");
    assert_eq!(instance.visible().red().paint("foo"), "");
    assert_eq!(instance.red().visible().paint("foo"), "");
    assert_eq!(instance.red().paint("foo"), "foo");

    instance.set_level(ColorSupportLevel::TrueColor);
    assert_eq!(instance.red().paint("foo"), "\u{1b}[31mfoo\u{1b}[39m");
    assert_eq!(
        instance.visible().red().paint("foo"),
        "\u{1b}[31mfoo\u{1b}[39m"
    );
    assert_eq!(
        instance.red().visible().paint("foo"),
        "\u{1b}[31mfoo\u{1b}[39m"
    );
    assert_eq!(instance.visible().paint("foo"), "foo");
    assert_eq!(instance.red().paint("foo"), "\u{1b}[31mfoo\u{1b}[39m");
}
