//! Port of `test/instance.js`.

use chalk::{Chalk, ColorSupportLevel, InvalidLevel, Options};

/// Every rejection in this file must carry the original's message.
fn assert_invalid_level<T>(result: &Result<T, InvalidLevel>, context: &str) {
    let Err(error) = result else {
        panic!("expected `{context}` to be rejected");
    };

    assert!(
        error
            .to_string()
            .contains("should be an integer from 0 to 3"),
        "unexpected message for `{context}`: {error}"
    );
}

/// `create an isolated context where colors can be disabled (by level)`
#[test]
fn create_an_isolated_context_where_colors_can_be_disabled_by_level() {
    // The original sets the shared instance's level here; an isolated instance
    // is used instead, because `cargo test` runs the tests in this file on
    // threads of one process rather than each in its own.
    let chalk = Chalk::with_level(ColorSupportLevel::Basic);

    let instance = Chalk::with_level(ColorSupportLevel::None);
    assert_eq!(instance.red().paint("foo"), "foo");
    assert_eq!(chalk.red().paint("foo"), "\u{1b}[31mfoo\u{1b}[39m");
    instance.set_level(ColorSupportLevel::Ansi256);
    assert_eq!(instance.red().paint("foo"), "\u{1b}[31mfoo\u{1b}[39m");
}

/// ``the `level` option should be a number from 0 to 3``
#[test]
fn the_level_option_should_be_a_number_from_0_to_3() {
    assert_invalid_level(&Chalk::try_with_level(10_i64), "level: 10");
    assert_invalid_level(&Chalk::try_with_level(-1_i64), "level: -1");
}

/// ``an omitted `level` option is detected rather than rejected``
#[test]
fn an_omitted_level_option_is_detected_rather_than_rejected() {
    assert_eq!(
        Chalk::with_options(Options { level: None }).level(),
        Chalk::new().level()
    );
}

/// ``assigning `level` is validated``
#[test]
fn assigning_level_is_validated() {
    let instance = Chalk::with_level(ColorSupportLevel::Basic);

    // Unlike the option, an absent value is not a way to ask for detection here.
    // The original's parameter set is `[10, -1, 1.5, ' 1', undefined]`; each is
    // rejected through the entry point that can express it.
    assert_invalid_level(&instance.try_set_level(10_i64), "level: 10");
    assert_invalid_level(&instance.try_set_level(-1_i64), "level: -1");
    assert_invalid_level(&instance.try_set_level(1.5_f64), "level: 1.5");
    assert_invalid_level(&" 1".parse::<ColorSupportLevel>(), "level: ' 1'");
    // `undefined` has no value-level spelling in Rust — the type system rejects
    // it before it can reach `try_set_level`. An empty string is the nearest
    // thing a caller can still pass, and it is rejected too.
    assert_invalid_level(&"".parse::<ColorSupportLevel>(), "level: undefined");

    // A style in the chain writes through to the instance, so it is validated too
    assert_invalid_level(&instance.red().try_set_level(10_i64), "red.level: 10");

    assert_eq!(instance.level(), ColorSupportLevel::Basic);

    instance.set_level(ColorSupportLevel::None);
    assert_eq!(instance.level(), ColorSupportLevel::None);
    assert_eq!(instance.red().paint("foo"), "foo");
}

/// `a cached model style keeps following the level`
#[test]
fn a_cached_model_style_keeps_following_the_level() {
    let instance = Chalk::with_level(ColorSupportLevel::TrueColor);

    assert_eq!(
        instance.rgb(255, 0, 0).paint("foo"),
        "\u{1b}[38;2;255;0;0mfoo\u{1b}[39m"
    );

    instance.set_level(ColorSupportLevel::Basic);
    assert_eq!(
        instance.rgb(255, 0, 0).paint("foo"),
        "\u{1b}[91mfoo\u{1b}[39m"
    );

    instance.set_level(ColorSupportLevel::None);
    assert_eq!(instance.rgb(255, 0, 0).paint("foo"), "foo");
}

/// `a model style cached on a style in the chain keeps following the level`
#[test]
fn a_model_style_cached_on_a_style_in_the_chain_keeps_following_the_level() {
    let instance = Chalk::with_level(ColorSupportLevel::TrueColor);
    let bold = instance.bold();

    assert_eq!(
        bold.rgb(255, 0, 0).paint("foo"),
        "\u{1b}[1m\u{1b}[38;2;255;0;0mfoo\u{1b}[39m\u{1b}[22m"
    );

    instance.set_level(ColorSupportLevel::Basic);
    assert_eq!(
        bold.rgb(255, 0, 0).paint("foo"),
        "\u{1b}[1m\u{1b}[91mfoo\u{1b}[39m\u{1b}[22m"
    );
}

/// `a deep chain reads the level from the instance it started on`
#[test]
fn a_deep_chain_reads_the_level_from_the_instance_it_started_on() {
    let instance = Chalk::with_level(ColorSupportLevel::Basic);
    let chain = instance.red().bold().underline();

    assert_eq!(chain.level(), ColorSupportLevel::Basic);

    instance.set_level(ColorSupportLevel::None);
    assert_eq!(chain.level(), ColorSupportLevel::None);
    assert_eq!(chain.paint("foo"), "foo");

    // Writing through the chain reaches the instance, however deep
    chain.set_level(ColorSupportLevel::Ansi256);
    assert_eq!(instance.level(), ColorSupportLevel::Ansi256);
}

// The tests below have no counterpart in `test/instance.js`. The original
// carries the level as a plain number, so its `level` accessor is exercised by
// every test in the suite; the port carries a `ColorSupportLevel` with explicit
// conversions at the boundary, and those conversions are new code that needs
// covering.

#[test]
fn a_level_converts_from_every_representation_a_caller_can_supply() {
    use std::convert::TryFrom;

    for (number, float, expected) in [
        (0_i64, 0.0_f64, ColorSupportLevel::None),
        (1, 1.0, ColorSupportLevel::Basic),
        (2, 2.0, ColorSupportLevel::Ansi256),
        (3, 3.0, ColorSupportLevel::TrueColor),
    ] {
        assert_eq!(ColorSupportLevel::try_from(number), Ok(expected));
        assert_eq!(ColorSupportLevel::try_from(float), Ok(expected));
        assert_eq!(number.to_string().parse(), Ok(expected));
        assert_eq!(expected.as_u8(), u8::try_from(number).unwrap());
        assert_eq!(expected.to_string(), number.to_string());
    }

    for number in [-1_i64, 4, 10, i64::MIN, i64::MAX] {
        assert_invalid_level(&ColorSupportLevel::try_from(number), "integer");
    }

    for number in [
        -1.0_f64,
        3.5,
        1.5,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert_invalid_level(&ColorSupportLevel::try_from(number), "float");
    }

    for text in ["", " 1", "1 ", "one", "1.0", "0x2", "4", "-1"] {
        assert_invalid_level(&text.parse::<ColorSupportLevel>(), text);
    }

    // An explicit sign is an ordinary integer spelling in Rust, and the
    // original never sees a string here at all, so there is nothing to reject.
    assert_eq!("+1".parse(), Ok(ColorSupportLevel::Basic));
}

#[test]
fn a_validated_level_that_is_in_range_is_assigned() {
    let instance = Chalk::with_level(ColorSupportLevel::None);

    instance.try_set_level(2_i64).expect("2 is in range");
    assert_eq!(instance.level(), ColorSupportLevel::Ansi256);

    instance.red().try_set_level(3_f64).expect("3 is in range");
    assert_eq!(instance.level(), ColorSupportLevel::TrueColor);

    assert_eq!(
        Chalk::try_with_level(1_i64).expect("1 is in range").level(),
        ColorSupportLevel::Basic
    );
}

#[test]
fn a_default_instance_is_a_detected_one() {
    assert_eq!(Chalk::default().level(), Chalk::new().level());
    assert_eq!(ColorSupportLevel::default(), ColorSupportLevel::None);
}
