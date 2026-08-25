//! Port of `test/no-color-support.js`.

use chalk::{ColorSupportLevel, chalk};

/// `colors can be forced by using chalk.level`
#[test]
fn colors_can_be_forced_by_using_chalk_level() {
    chalk().set_level(ColorSupportLevel::Basic);
    assert_eq!(chalk().green().paint("hello"), "\u{1b}[32mhello\u{1b}[39m");
}
