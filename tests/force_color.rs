//! Port of `test/force-color.js`.

use std::process::Command;

/// Run the fixture and return what it printed.
///
/// The environment is not extended so that the ambient `CI`, `TERM`, and
/// `COLORTERM` of the machine running the tests cannot leak into the fixture.
fn detect_level(environment: &[(&str, &str)], flags: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_force_color_fixture"))
        .args(flags)
        .env_clear()
        .envs(environment.iter().copied())
        .output()
        .expect("the fixture should run");

    assert!(output.status.success(), "the fixture should exit cleanly");

    String::from_utf8(output.stdout)
        .expect("the fixture should print UTF-8")
        .trim_end_matches(['\r', '\n'])
        .to_owned()
}

/// ``  `FORCE_COLOR=1` is an exact level, not a minimum ``
#[test]
fn force_color_1_is_an_exact_level_not_a_minimum() {
    assert_eq!(
        detect_level(&[("FORCE_COLOR", "1"), ("COLORTERM", "truecolor")], &[]),
        "1"
    );
}

/// ``  `FORCE_COLOR=2` is an exact level, not a minimum ``
#[test]
fn force_color_2_is_an_exact_level_not_a_minimum() {
    assert_eq!(
        detect_level(&[("FORCE_COLOR", "2"), ("COLORTERM", "truecolor")], &[]),
        "2"
    );
}

/// ``  `FORCE_COLOR=3` enables truecolor ``
#[test]
fn force_color_3_enables_truecolor() {
    assert_eq!(detect_level(&[("FORCE_COLOR", "3")], &[]), "3");
}

/// ``  `FORCE_COLOR=1` overrides CI detection ``
#[test]
fn force_color_1_overrides_ci_detection() {
    assert_eq!(
        detect_level(
            &[
                ("FORCE_COLOR", "1"),
                ("CI", "true"),
                ("GITHUB_ACTIONS", "true")
            ],
            &[]
        ),
        "1"
    );
}

/// ``  `FORCE_COLOR=1` overrides TERM detection ``
#[test]
fn force_color_1_overrides_term_detection() {
    assert_eq!(
        detect_level(&[("FORCE_COLOR", "1"), ("TERM", "xterm-256color")], &[]),
        "1"
    );
}

/// `` a `FORCE_COLOR` above 3 is clamped to 3 ``
#[test]
fn a_force_color_above_3_is_clamped_to_3() {
    assert_eq!(
        detect_level(&[("FORCE_COLOR", "4"), ("TERM", "xterm-256color")], &[]),
        "3"
    );
}

/// `` the `--color=256` and `--color=16m` flags take precedence over a numeric `FORCE_COLOR` ``
#[test]
fn the_color_flags_take_precedence_over_a_numeric_force_color() {
    assert_eq!(detect_level(&[("FORCE_COLOR", "1")], &["--color=256"]), "2");
    assert_eq!(detect_level(&[("FORCE_COLOR", "1")], &["--color=16m"]), "3");
}

/// ``  `FORCE_COLOR=0` disables color ``
#[test]
fn force_color_0_disables_color() {
    assert_eq!(
        detect_level(&[("FORCE_COLOR", "0"), ("COLORTERM", "truecolor")], &[]),
        "0"
    );
}

/// ``  `FORCE_COLOR=true` only enables color and lets the level be detected ``
#[test]
fn force_color_true_only_enables_color_and_lets_the_level_be_detected() {
    assert_eq!(
        detect_level(&[("FORCE_COLOR", "true"), ("COLORTERM", "truecolor")], &[]),
        "3"
    );
    assert_eq!(
        detect_level(&[("FORCE_COLOR", "true"), ("TERM", "xterm-256color")], &[]),
        "2"
    );
    assert_eq!(detect_level(&[("FORCE_COLOR", "true")], &[]), "1");
}

/// `` an empty `FORCE_COLOR` behaves like `FORCE_COLOR=true` ``
#[test]
fn an_empty_force_color_behaves_like_force_color_true() {
    assert_eq!(
        detect_level(&[("FORCE_COLOR", ""), ("COLORTERM", "truecolor")], &[]),
        "3"
    );
}

/// ``  `FORCE_COLOR=false` disables color ``
#[test]
fn force_color_false_disables_color() {
    assert_eq!(
        detect_level(&[("FORCE_COLOR", "false"), ("COLORTERM", "truecolor")], &[]),
        "0"
    );
}

/// `` a non-numeric `FORCE_COLOR` is treated as unset, not as disabled ``
#[test]
fn a_non_numeric_force_color_is_treated_as_unset_not_as_disabled() {
    assert_eq!(
        detect_level(
            &[("FORCE_COLOR", "unicorn"), ("COLORTERM", "truecolor")],
            &[]
        ),
        "0"
    );

    // The fixture is piped, so unset and disabled both give 0. The Azure check sits above the non-TTY check, so it tells the two apart.
    assert_eq!(
        detect_level(
            &[
                ("FORCE_COLOR", "unicorn"),
                ("TF_BUILD", "1"),
                ("AGENT_NAME", "agent")
            ],
            &[]
        ),
        "1"
    );
    assert_eq!(
        detect_level(
            &[
                ("FORCE_COLOR", "0"),
                ("TF_BUILD", "1"),
                ("AGENT_NAME", "agent")
            ],
            &[]
        ),
        "0"
    );
}

/// `` a partly numeric `FORCE_COLOR` is treated as unset, not as a level ``
#[test]
fn a_partly_numeric_force_color_is_treated_as_unset_not_as_a_level() {
    for force_color in [" 2", "2 ", "2abc", "+2", "1e1", "0x2"] {
        let piped = detect_level(
            &[("FORCE_COLOR", force_color), ("COLORTERM", "truecolor")],
            &[],
        );
        let azure = detect_level(
            &[
                ("FORCE_COLOR", force_color),
                ("TF_BUILD", "1"),
                ("AGENT_NAME", "agent"),
            ],
            &[],
        );

        assert_eq!(piped, "0", "FORCE_COLOR={force_color}");
        assert_eq!(azure, "1", "FORCE_COLOR={force_color}");
    }
}
