//! Generates the screenshot in the readme.
//!
//! Run with `cargo run --example screenshot`.

use std::io::{self, Write};

use chalk::{BACKGROUND_COLOR_NAMES, FOREGROUND_COLOR_NAMES, MODIFIER_NAMES, chalk};

fn main() {
    let mut stdout = io::stdout();

    for key in MODIFIER_NAMES
        .iter()
        .chain(FOREGROUND_COLOR_NAMES)
        .chain(BACKGROUND_COLOR_NAMES)
    {
        let mut return_value = (*key).to_owned();

        // We skip `overline` as almost no terminal supports it so we cannot show it off.
        if matches!(
            *key,
            "reset" | "hidden" | "grey" | "bgGray" | "bgGrey" | "overline"
        ) || key.ends_with("Bright")
        {
            continue;
        }

        // A dark background needs dark text on it to be readable.
        if key.starts_with("bg") && !key["bg".len()..].starts_with('B') {
            return_value = chalk().black().paint(return_value);
        }

        let styled = chalk()
            .style_by_name(key)
            .expect("every listed name is a style")
            .paint(return_value);

        let _ = write!(stdout, "{styled} ");
    }

    let _ = writeln!(stdout);
}
