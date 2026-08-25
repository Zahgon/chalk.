//! Tests for the vendored `ansi-styles` module.
//!
//! The original excludes `source/vendor` from its coverage report: this code
//! was covered by the upstream packages' own suites, not by chalk's. Vendored
//! into the port it is no longer anyone else's, so it is covered here.

use chalk::vendor::ansi_styles::{
    BACKGROUND_COLOR_NAMES, CODES, ColorType, FOREGROUND_COLOR_NAMES, MODIFIER_NAMES,
    UNDERLINE_COLOR_NAMES, ansi256_to_ansi, hex_to_ansi, hex_to_ansi256, hex_to_rgb, rgb_to_ansi,
    rgb_to_ansi256, style,
};
use chalk::{COLOR_NAMES, Chalk, ColorSupportLevel};

#[test]
fn every_named_style_wraps_its_parameter_in_an_sgr_sequence() {
    for name in MODIFIER_NAMES
        .iter()
        .chain(FOREGROUND_COLOR_NAMES)
        .chain(BACKGROUND_COLOR_NAMES)
        .chain(UNDERLINE_COLOR_NAMES)
    {
        let pair = style(name).unwrap_or_else(|| panic!("`{name}` should be a known style"));

        for sequence in [pair.open, pair.close] {
            assert!(
                sequence.starts_with("\u{1b}[") && sequence.ends_with('m'),
                "`{name}` produced {sequence:?}"
            );
        }
    }

    assert_eq!(style("nope"), None);
}

#[test]
fn the_style_name_lists_are_in_declaration_order() {
    assert_eq!(MODIFIER_NAMES[0], "reset");
    assert_eq!(MODIFIER_NAMES[MODIFIER_NAMES.len() - 1], "strikethrough");
    assert_eq!(FOREGROUND_COLOR_NAMES[0], "black");
    assert_eq!(BACKGROUND_COLOR_NAMES[0], "bgBlack");
    assert_eq!(UNDERLINE_COLOR_NAMES[0], "underlineBlack");

    // `colorNames` is the foreground names followed by the background ones, and
    // deliberately excludes the underline colours.
    let expected: Vec<&str> = FOREGROUND_COLOR_NAMES
        .iter()
        .chain(BACKGROUND_COLOR_NAMES)
        .copied()
        .collect();
    assert_eq!(*COLOR_NAMES, expected);
    for name in UNDERLINE_COLOR_NAMES {
        assert!(!COLOR_NAMES.contains(name));
    }
}

#[test]
fn aliases_produce_the_same_sequences_as_what_they_alias() {
    for (alias, aliased) in [
        ("gray", "blackBright"),
        ("grey", "blackBright"),
        ("bgGray", "bgBlackBright"),
        ("bgGrey", "bgBlackBright"),
        ("underlineGray", "underlineBlackBright"),
        ("underlineGrey", "underlineBlackBright"),
    ] {
        assert_eq!(style(alias), style(aliased), "{alias} vs {aliased}");
    }
}

#[test]
fn codes_key_a_style_by_the_leading_parameter_of_its_open_sequence() {
    let codes: std::collections::HashMap<u16, u16> = CODES.iter().copied().collect();

    // `4:3` is keyed as `4`, and `58;5;1` as `58`.
    assert_eq!(codes.get(&4), Some(&24));
    assert_eq!(codes.get(&58), Some(&59));
    assert_eq!(codes.get(&1), Some(&22));
    assert_eq!(codes.get(&2), Some(&22));
    assert_eq!(codes.get(&31), Some(&39));
    assert_eq!(codes.get(&41), Some(&49));
    assert_eq!(codes.get(&53), Some(&55));

    // Insertion order is preserved and keys are not repeated.
    assert_eq!(CODES[0], (0, 0));
    let mut seen = std::collections::HashSet::new();
    for (key, _) in CODES.iter() {
        assert!(seen.insert(*key), "duplicate key {key}");
    }
}

#[test]
fn the_colour_wrappers_offset_the_foreground_parameters() {
    assert_eq!(ColorType::Foreground.close(), "\u{1b}[39m");
    assert_eq!(ColorType::Background.close(), "\u{1b}[49m");
    assert_eq!(ColorType::Underline.close(), "\u{1b}[59m");

    assert_eq!(ColorType::Foreground.ansi(31), "\u{1b}[31m");
    assert_eq!(ColorType::Background.ansi(31), "\u{1b}[41m");
    assert_eq!(ColorType::Foreground.ansi(91), "\u{1b}[91m");
    assert_eq!(ColorType::Background.ansi(91), "\u{1b}[101m");

    assert_eq!(ColorType::Foreground.ansi256(196), "\u{1b}[38;5;196m");
    assert_eq!(ColorType::Background.ansi256(196), "\u{1b}[48;5;196m");
    assert_eq!(ColorType::Underline.ansi256(196), "\u{1b}[58;5;196m");

    assert_eq!(ColorType::Foreground.ansi16m(1, 2, 3), "\u{1b}[38;2;1;2;3m");
    assert_eq!(ColorType::Background.ansi16m(1, 2, 3), "\u{1b}[48;2;1;2;3m");
    assert_eq!(ColorType::Underline.ansi16m(1, 2, 3), "\u{1b}[58;2;1;2;3m");
}

#[test]
fn the_underline_ansi_wrapper_maps_a_basic_code_to_a_palette_index() {
    // `SGR 58` has no basic 16-colour form, so `30..=37` become `0..=7` and
    // `90..=97` become `8..=15`.
    assert_eq!(ColorType::Underline.ansi(30), "\u{1b}[58;5;0m");
    assert_eq!(ColorType::Underline.ansi(37), "\u{1b}[58;5;7m");
    assert_eq!(ColorType::Underline.ansi(90), "\u{1b}[58;5;8m");
    assert_eq!(ColorType::Underline.ansi(97), "\u{1b}[58;5;15m");
}

#[test]
fn rgb_to_ansi256_uses_the_extended_greyscale_palette() {
    assert_eq!(rgb_to_ansi256(0, 0, 0), 16);
    assert_eq!(rgb_to_ansi256(7, 7, 7), 16);
    assert_eq!(rgb_to_ansi256(8, 8, 8), 232);
    assert_eq!(rgb_to_ansi256(128, 128, 128), 244);
    assert_eq!(rgb_to_ansi256(248, 248, 248), 255);
    assert_eq!(rgb_to_ansi256(249, 249, 249), 231);
    assert_eq!(rgb_to_ansi256(255, 255, 255), 231);
}

#[test]
fn rgb_to_ansi256_maps_the_colour_cube() {
    assert_eq!(rgb_to_ansi256(255, 0, 0), 196);
    assert_eq!(rgb_to_ansi256(0, 255, 0), 46);
    assert_eq!(rgb_to_ansi256(0, 0, 255), 21);
    assert_eq!(rgb_to_ansi256(222, 173, 237), 183);
}

#[test]
fn ansi256_to_ansi_maps_the_first_sixteen_directly() {
    assert_eq!(ansi256_to_ansi(0), 30);
    assert_eq!(ansi256_to_ansi(2), 32);
    assert_eq!(ansi256_to_ansi(7), 37);
    assert_eq!(ansi256_to_ansi(8), 90);
    assert_eq!(ansi256_to_ansi(15), 97);
}

#[test]
fn ansi256_to_ansi_downsamples_the_cube_and_the_greyscale_ramp() {
    assert_eq!(ansi256_to_ansi(16), 30);
    assert_eq!(ansi256_to_ansi(21), 94);
    assert_eq!(ansi256_to_ansi(46), 92);
    assert_eq!(ansi256_to_ansi(196), 91);
    assert_eq!(ansi256_to_ansi(231), 97);
    assert_eq!(ansi256_to_ansi(232), 30);
    assert_eq!(ansi256_to_ansi(255), 37);
}

#[test]
fn hex_to_rgb_takes_the_leftmost_run_of_hex_digits() {
    assert_eq!(hex_to_rgb("#FF0000"), (255, 0, 0));
    assert_eq!(hex_to_rgb("#ff0000"), (255, 0, 0));
    assert_eq!(hex_to_rgb("FF0000"), (255, 0, 0));
    assert_eq!(hex_to_rgb("#DEADED"), (0xDE, 0xAD, 0xED));
    assert_eq!(hex_to_rgb("abcdefg"), (0xAB, 0xCD, 0xEF));

    // A non-ASCII byte is not a hex digit and never splits a character: the
    // run stops at it, and a three-digit run is still a match.
    assert_eq!(hex_to_rgb("caf\u{e9}"), (0xCC, 0xAA, 0xFF));
    assert_eq!(hex_to_rgb("\u{2192}#00ff00"), (0, 255, 0));
}

#[test]
fn hex_to_rgb_expands_a_three_digit_match() {
    assert_eq!(hex_to_rgb("#f0a"), (0xFF, 0x00, 0xAA));
    assert_eq!(hex_to_rgb("#FFF"), (255, 255, 255));
}

#[test]
fn hex_to_rgb_is_black_when_nothing_matches() {
    assert_eq!(hex_to_rgb(""), (0, 0, 0));
    assert_eq!(hex_to_rgb("#12"), (0, 0, 0));
    assert_eq!(hex_to_rgb("nope"), (0, 0, 0));
    assert_eq!(hex_to_rgb("\u{2192}\u{2192}"), (0, 0, 0));
}

#[test]
fn the_composed_conversions_agree_with_their_parts() {
    for hex in ["#FF0000", "#00ff00", "#0000FF", "#deaded", "#fff", "nope"] {
        let (red, green, blue) = hex_to_rgb(hex);
        assert_eq!(
            hex_to_ansi256(hex),
            rgb_to_ansi256(red, green, blue),
            "{hex}"
        );
        assert_eq!(hex_to_ansi(hex), rgb_to_ansi(red, green, blue), "{hex}");
    }

    // Every palette index downsamples to a basic colour code, and never to
    // anything outside the two basic ranges.
    for code in 0..=255_u8 {
        let basic = ansi256_to_ansi(code);
        assert!(
            (30..=37).contains(&basic) || (90..=97).contains(&basic),
            "ansi256_to_ansi({code}) produced {basic}"
        );
    }

    // `rgb_to_ansi` is `ansi256_to_ansi` composed onto `rgb_to_ansi256`.
    for (red, green, blue) in [(255, 0, 0), (0, 255, 0), (0, 0, 255), (8, 8, 8), (0, 0, 0)] {
        assert_eq!(
            rgb_to_ansi(red, green, blue),
            ansi256_to_ansi(rgb_to_ansi256(red, green, blue))
        );
    }
}

#[test]
fn every_named_style_is_reachable_through_style_by_name() {
    let chalk = Chalk::with_level(ColorSupportLevel::TrueColor);

    for name in MODIFIER_NAMES
        .iter()
        .chain(FOREGROUND_COLOR_NAMES)
        .chain(BACKGROUND_COLOR_NAMES)
        .chain(UNDERLINE_COLOR_NAMES)
    {
        let pair = style(name).unwrap();
        let styled = chalk
            .style_by_name(name)
            .unwrap_or_else(|| panic!("`{name}` should be reachable"))
            .paint("x");

        assert_eq!(styled, format!("{}x{}", pair.open, pair.close), "{name}");
    }

    assert!(chalk.style_by_name("nope").is_none());
}
