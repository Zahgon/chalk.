//! Tests for the vendored `supports-color` module.
//!
//! The original can only reach this code through a spawned child process with a
//! controlled environment, which is why `test/force-color.js` exercises only the
//! `FORCE_COLOR` branches — and why `source/vendor` is excluded from the
//! original's coverage report altogether. Here the decision procedure is a pure
//! function of its inputs, so every branch of it is reachable in-process.

use chalk::vendor::supports_color::{
    Inputs, Options, SUPPORTS_COLOR, create_supports_color, create_supports_color_from,
    detect_level,
};
use chalk::{chalk_stderr, supports_color, supports_color_stderr};

/// The inputs for a detection, with no environment and no arguments.
fn inputs(environment: &[(&str, &str)]) -> Inputs {
    Inputs {
        env: environment
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect(),
        args: vec!["program".to_owned()],
        windows_release: None,
    }
}

/// The level detected for a TTY with this environment.
fn level(environment: &[(&str, &str)]) -> u8 {
    detect_level(&inputs(environment), Some(true), true)
}

/// The level detected for a piped stream with this environment.
fn piped_level(environment: &[(&str, &str)]) -> u8 {
    detect_level(&inputs(environment), Some(false), true)
}

/// The level detected for a TTY with this environment and these arguments.
fn level_with_flags(environment: &[(&str, &str)], flags: &[&str]) -> u8 {
    let mut inputs = inputs(environment);
    inputs
        .args
        .extend(flags.iter().map(|flag| (*flag).to_owned()));
    detect_level(&inputs, Some(true), true)
}

#[test]
fn a_flag_only_counts_before_a_bare_terminator() {
    assert_eq!(level_with_flags(&[], &["--color=256"]), 2);
    assert_eq!(level_with_flags(&[], &["--", "--color=256"]), 0);
    assert_eq!(level_with_flags(&[], &["--color=256", "--"]), 2);
    // A single-character flag takes one dash, and a flag that already has one
    // is used verbatim.
    assert_eq!(level_with_flags(&[], &["--no-color"]), 0);
    assert_eq!(level_with_flags(&[], &["--color"]), 1);
    assert_eq!(level_with_flags(&[], &["--colors"]), 1);
    assert_eq!(level_with_flags(&[], &["--color=always"]), 1);
    assert_eq!(level_with_flags(&[], &["--color=true"]), 1);
    assert_eq!(level_with_flags(&[], &["--no-colors"]), 0);
    assert_eq!(level_with_flags(&[], &["--color=false"]), 0);
    assert_eq!(level_with_flags(&[], &["--color=never"]), 0);
    assert_eq!(level_with_flags(&[], &["--color=16m"]), 3);
    assert_eq!(level_with_flags(&[], &["--color=full"]), 3);
    assert_eq!(level_with_flags(&[], &["--color=truecolor"]), 3);
}

#[test]
fn flags_are_ignored_when_sniffing_is_off() {
    let mut inputs = inputs(&[]);
    inputs.args.push("--color=16m".to_owned());

    assert_eq!(detect_level(&inputs, Some(true), true), 3);
    assert_eq!(detect_level(&inputs, Some(true), false), 0);

    // `FORCE_COLOR` still applies without flag sniffing.
    let mut forced = inputs.clone();
    forced.env.insert("FORCE_COLOR".to_owned(), "2".to_owned());
    assert_eq!(detect_level(&forced, Some(true), false), 2);
}

#[test]
fn the_azure_check_sits_above_the_tty_check() {
    assert_eq!(piped_level(&[("TF_BUILD", "1"), ("AGENT_NAME", "a")]), 1);
    // Both variables are required.
    assert_eq!(piped_level(&[("TF_BUILD", "1")]), 0);
    assert_eq!(piped_level(&[("AGENT_NAME", "a")]), 0);
}

#[test]
fn a_stream_that_is_not_a_tty_disables_colour_unless_it_is_forced() {
    assert_eq!(piped_level(&[("COLORTERM", "truecolor")]), 0);
    assert_eq!(
        piped_level(&[("COLORTERM", "truecolor"), ("FORCE_COLOR", "true")]),
        3
    );
    // With no stream at all there is nothing to be a TTY, and detection carries on.
    assert_eq!(
        detect_level(&inputs(&[("COLORTERM", "truecolor")]), None, true),
        3
    );
}

#[test]
fn a_dumb_terminal_reports_only_what_is_forced() {
    assert_eq!(level(&[("TERM", "dumb")]), 0);
    assert_eq!(level(&[("TERM", "dumb"), ("COLORTERM", "truecolor")]), 0);
    assert_eq!(level(&[("TERM", "dumb"), ("FORCE_COLOR", "true")]), 1);
}

#[test]
fn continuous_integration_services_are_recognised() {
    for key in ["GITHUB_ACTIONS", "GITEA_ACTIONS", "CIRCLECI"] {
        assert_eq!(level(&[("CI", "true"), (key, "1")]), 3, "{key}");
    }

    for key in ["TRAVIS", "APPVEYOR", "GITLAB_CI", "BUILDKITE", "DRONE"] {
        assert_eq!(level(&[("CI", "true"), (key, "1")]), 1, "{key}");
    }

    assert_eq!(level(&[("CI", "true"), ("CI_NAME", "codeship")]), 1);
    assert_eq!(level(&[("CI", "true"), ("CI_NAME", "other")]), 0);

    // An unrecognised CI reports the forced minimum, and the CI branch wins
    // over everything below it.
    assert_eq!(level(&[("CI", "true"), ("COLORTERM", "truecolor")]), 0);
    assert_eq!(
        level(&[("CI", "true"), ("FORCE_COLOR", "true"), ("TERM", "dumb")]),
        1
    );
}

#[test]
fn teamcity_needs_a_version_of_at_least_9_1() {
    for version in ["9.1.5", "9.01.5", "10.0.1", "2019.1", "9.12.3"] {
        assert_eq!(level(&[("TEAMCITY_VERSION", version)]), 1, "{version}");
    }

    for version in ["9.0.5", "9.0.0", "8.1", "9", "9.", "9.1", "abc", ""] {
        assert_eq!(level(&[("TEAMCITY_VERSION", version)]), 0, "{version}");
    }
}

#[test]
fn truecolor_terminals_are_recognised() {
    assert_eq!(level(&[("COLORTERM", "truecolor")]), 3);
    assert_eq!(level(&[("TERM", "xterm-kitty")]), 3);
    assert_eq!(level(&[("TERM", "xterm-ghostty")]), 3);
    assert_eq!(level(&[("TERM", "wezterm")]), 3);
}

#[test]
fn term_program_is_consulted_before_term() {
    assert_eq!(
        level(&[
            ("TERM_PROGRAM", "iTerm.app"),
            ("TERM_PROGRAM_VERSION", "3.0.10")
        ]),
        3
    );
    assert_eq!(
        level(&[
            ("TERM_PROGRAM", "iTerm.app"),
            ("TERM_PROGRAM_VERSION", "2.9.7")
        ]),
        2
    );
    // A missing or unparsable version is treated as older than 3.
    assert_eq!(level(&[("TERM_PROGRAM", "iTerm.app")]), 2);
    assert_eq!(
        level(&[("TERM_PROGRAM", "iTerm.app"), ("TERM_PROGRAM_VERSION", "x")]),
        2
    );
    assert_eq!(
        level(&[("TERM_PROGRAM", "iTerm.app"), ("TERM_PROGRAM_VERSION", "")]),
        2
    );

    // The original reads the version with `Number.parseInt`, which takes the
    // leading integer and ignores any trailing text or leading whitespace. A
    // strict parse here would report 2 for each of these.
    for version in ["3abc", " 3", "3-beta", "3.4.19-nightly", "\t4", "+3", "12"] {
        assert_eq!(
            level(&[
                ("TERM_PROGRAM", "iTerm.app"),
                ("TERM_PROGRAM_VERSION", version)
            ]),
            3,
            "TERM_PROGRAM_VERSION={version:?} should read as major version >= 3"
        );
    }

    // A leading integer below 3 still loses, however it is spelled.
    for version in ["2abc", " 2", "-3", "0002"] {
        assert_eq!(
            level(&[
                ("TERM_PROGRAM", "iTerm.app"),
                ("TERM_PROGRAM_VERSION", version)
            ]),
            2,
            "TERM_PROGRAM_VERSION={version:?} should read as major version < 3"
        );
    }
    assert_eq!(level(&[("TERM_PROGRAM", "Apple_Terminal")]), 2);

    // An unknown terminal program falls through to the `TERM` checks.
    assert_eq!(
        level(&[("TERM_PROGRAM", "Hyper"), ("TERM", "xterm-256color")]),
        2
    );
    assert_eq!(level(&[("TERM_PROGRAM", "Hyper")]), 0);
}

#[test]
fn term_is_matched_for_256_colour_and_then_basic_colour() {
    for term in ["xterm-256color", "screen-256", "SCREEN-256COLOR"] {
        assert_eq!(level(&[("TERM", term)]), 2, "{term}");
    }

    for term in [
        "screen",
        "xterm",
        "vt100",
        "vt220",
        "rxvt",
        "linux",
        "cygwin",
        "konsole-ansi",
        "Eterm-color",
    ] {
        assert_eq!(level(&[("TERM", term)]), 1, "{term}");
    }

    assert_eq!(level(&[("TERM", "foo")]), 0);
    assert_eq!(level(&[]), 0);
}

#[test]
fn any_colorterm_at_all_implies_basic_colour() {
    assert_eq!(level(&[("COLORTERM", "")]), 1);
    assert_eq!(level(&[("COLORTERM", "24bit")]), 1);
}

#[test]
fn force_color_is_interpreted_exactly() {
    assert_eq!(level(&[("FORCE_COLOR", "false")]), 0);
    assert_eq!(level(&[("FORCE_COLOR", "0")]), 0);
    assert_eq!(level(&[("FORCE_COLOR", "1")]), 1);
    assert_eq!(level(&[("FORCE_COLOR", "2")]), 2);
    assert_eq!(level(&[("FORCE_COLOR", "3")]), 3);
    assert_eq!(level(&[("FORCE_COLOR", "4")]), 3);
    // Numeric but far too large to fit an integer: still numeric, still clamped.
    assert_eq!(level(&[("FORCE_COLOR", "99999999999999999999999")]), 3);

    // `true` and the empty string only enable colour; the level is detected.
    assert_eq!(
        level(&[("FORCE_COLOR", "true"), ("COLORTERM", "truecolor")]),
        3
    );
    assert_eq!(level(&[("FORCE_COLOR", ""), ("TERM", "xterm-256color")]), 2);

    // Anything else is as good as unset.
    for value in ["unicorn", " 2", "2 ", "2abc", "+2", "1e1", "0x2", "-1"] {
        assert_eq!(
            level(&[("FORCE_COLOR", value), ("TERM", "xterm-256color")]),
            2,
            "{value}"
        );
    }

    // `FORCE_COLOR` wins over the flags.
    assert_eq!(level_with_flags(&[("FORCE_COLOR", "0")], &["--color"]), 0);
    assert_eq!(
        level_with_flags(&[("FORCE_COLOR", "2")], &["--no-color"]),
        2
    );
}

#[test]
fn the_reported_shape_describes_the_level() {
    let options = Options::default();

    assert_eq!(
        create_supports_color_from(&inputs(&[]), Some(false), options),
        None
    );

    for (value, expected_256, expected_16m) in
        [("1", false, false), ("2", true, false), ("3", true, true)]
    {
        let support =
            create_supports_color_from(&inputs(&[("FORCE_COLOR", value)]), Some(false), options)
                .unwrap_or_else(|| panic!("FORCE_COLOR={value} should report support"));

        assert_eq!(support.level.as_u8().to_string(), value);
        assert!(support.has_basic);
        assert_eq!(support.has_256, expected_256, "FORCE_COLOR={value}");
        assert_eq!(support.has_16m, expected_16m, "FORCE_COLOR={value}");
    }
}

#[test]
fn a_windows_release_selects_the_level_on_windows() {
    let with_release = |release: &str| {
        let mut inputs = inputs(&[]);
        inputs.windows_release = Some(release.to_owned());
        detect_level(&inputs, Some(true), true)
    };

    // Build 10586 is the first that supports 256 colours, 14931 the first that
    // supports Truecolor.
    assert_eq!(with_release("10.0.10585"), 1);
    assert_eq!(with_release("10.0.10586"), 2);
    assert_eq!(with_release("10.0.14930"), 2);
    assert_eq!(with_release("10.0.14931"), 3);
    assert_eq!(with_release("10.0.19045"), 3);
    assert_eq!(with_release("6.1.7601"), 1);
    // A release string that cannot be read falls back to basic colour.
    assert_eq!(with_release("10.0"), 1);
    assert_eq!(with_release("abc.0.19045"), 1);
    assert_eq!(with_release(""), 1);
}

#[test]
fn a_flag_is_spelled_by_its_length_unless_it_is_already_dashed() {
    let inputs = Inputs {
        env: std::collections::HashMap::new(),
        args: vec![
            "program".to_owned(),
            "-c".to_owned(),
            "--color=256".to_owned(),
            "-x".to_owned(),
        ],
        windows_release: None,
    };

    // Two or more characters take `--`.
    assert!(inputs.has_flag("color=256"));
    assert!(!inputs.has_flag("color=16m"));

    // One character takes `-`.
    assert!(inputs.has_flag("c"));
    assert!(inputs.has_flag("x"));
    assert!(!inputs.has_flag("y"));

    // A flag that already starts with a dash is used verbatim.
    assert!(inputs.has_flag("-c"));
    assert!(inputs.has_flag("--color=256"));
    assert!(!inputs.has_flag("-color=256"));
}

#[test]
fn detection_can_read_the_running_process() {
    // The test binary's stdout is not a terminal under `cargo test`, and the
    // ambient environment is whatever the developer has; all that can be
    // asserted without pinning the machine is that the two entry points agree
    // and stay inside the documented range.
    let options = Options::default();
    let from_process = create_supports_color(Some(false), options);
    let from_inputs = create_supports_color_from(&Inputs::from_process(), Some(false), options);

    assert_eq!(from_process, from_inputs);

    if let Some(support) = from_process {
        assert!(support.has_basic);
        assert_eq!(support.has_256, support.level.as_u8() >= 2);
        assert_eq!(support.has_16m, support.level.as_u8() >= 3);
    }

    // The two exported instances read the same detection as the module.
    assert_eq!(supports_color(), SUPPORTS_COLOR.stdout);
    assert_eq!(supports_color_stderr(), SUPPORTS_COLOR.stderr);
    assert_eq!(
        chalk_stderr().level().as_u8(),
        SUPPORTS_COLOR
            .stderr
            .map_or(0, |support| support.level.as_u8())
    );
}
