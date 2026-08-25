//! Port of `test/level.js`.

use std::process::Command;
use std::sync::{Mutex, MutexGuard};

use chalk::{ColorSupportLevel, chalk};

/// The tests below mutate the shared instance's level. The original relies on
/// AVA giving each test *file* its own process and on each test restoring the
/// level it found; `cargo test` runs the tests in a file on threads of one
/// process, so they take turns here instead.
static SHARED_INSTANCE: Mutex<()> = Mutex::new(());

/// Take the shared instance for the duration of a test.
fn shared_instance() -> MutexGuard<'static, ()> {
    SHARED_INSTANCE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// `don't output colors when manually disabled`
#[test]
fn dont_output_colors_when_manually_disabled() {
    let _guard = shared_instance();
    chalk().set_level(ColorSupportLevel::Basic);

    let old_level = chalk().level();
    chalk().set_level(ColorSupportLevel::None);
    assert_eq!(chalk().red().paint("foo"), "foo");
    chalk().set_level(old_level);
}

/// `enable/disable colors based on overall chalk .level property, not individual instances`
#[test]
fn enable_disable_colors_based_on_overall_chalk_level_property() {
    let _guard = shared_instance();
    chalk().set_level(ColorSupportLevel::Basic);

    let old_level = chalk().level();
    chalk().set_level(ColorSupportLevel::Basic);
    let red = chalk().red();
    assert_eq!(red.level(), ColorSupportLevel::Basic);
    chalk().set_level(ColorSupportLevel::None);
    assert_eq!(red.level(), chalk().level());
    chalk().set_level(old_level);
}

/// `propagate enable/disable changes from child colors`
#[test]
fn propagate_enable_disable_changes_from_child_colors() {
    let _guard = shared_instance();
    chalk().set_level(ColorSupportLevel::Basic);

    let old_level = chalk().level();
    chalk().set_level(ColorSupportLevel::Basic);
    let red = chalk().red();
    assert_eq!(red.level(), ColorSupportLevel::Basic);
    assert_eq!(chalk().level(), ColorSupportLevel::Basic);
    red.set_level(ColorSupportLevel::None);
    assert_eq!(red.level(), ColorSupportLevel::None);
    assert_eq!(chalk().level(), ColorSupportLevel::None);
    chalk().set_level(ColorSupportLevel::Basic);
    assert_eq!(red.level(), ColorSupportLevel::Basic);
    assert_eq!(chalk().level(), ColorSupportLevel::Basic);
    chalk().set_level(old_level);
}

/// `disable colors if they are not supported`
#[test]
fn disable_colors_if_they_are_not_supported() {
    let output = Command::new(env!("CARGO_BIN_EXE_fixture"))
        .output()
        .expect("the fixture should run");

    assert!(output.status.success(), "the fixture should exit cleanly");

    let stdout = String::from_utf8(output.stdout).expect("the fixture should print UTF-8");
    assert_eq!(stdout.trim_end_matches(['\r', '\n']), "testout testerr");
}
