//! Animates a rainbow-coloured string in place.
//!
//! Run with `cargo run --example rainbow`.
//!
//! The original leans on two development dependencies for this — `color-convert`
//! for HSL, and `log-update` for redrawing a line. Both are a handful of lines,
//! and the library itself has no dependencies, so they are inlined here rather
//! than pulled in.

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use chalk::chalk;

/// Characters outside printable ASCII are passed through uncoloured.
fn is_ignored(character: char) -> bool {
    !('\u{0021}'..='\u{007E}').contains(&character)
}

/// Convert an HSL colour to the `#RRGGBB` string Chalk's `hex` takes.
///
/// From <https://github.com/Qix-/color-convert>.
fn hsl_to_hex(hue: f64, saturation: f64, lightness: f64) -> String {
    let hue = hue / 360.0;
    let saturation = saturation / 100.0;
    let lightness = lightness / 100.0;

    let channel = |value: f64| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the value is a colour channel already scaled to 0..=255"
        )]
        let byte = (value * 255.0).round() as u8;
        byte
    };

    if saturation == 0.0 {
        let grey = channel(lightness);
        return format!("#{grey:02X}{grey:02X}{grey:02X}");
    }

    let upper = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - lightness * saturation
    };
    let lower = 2.0f64.mul_add(lightness, -upper);

    let mut rgb = [0u8; 3];
    for (index, slot) in rgb.iter_mut().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "the index is 0, 1, or 2")]
        let mut third = hue + 1.0 / 3.0 * -(index as f64 - 1.0);

        if third < 0.0 {
            third += 1.0;
        }
        if third > 1.0 {
            third -= 1.0;
        }

        let value = if 6.0 * third < 1.0 {
            (upper - lower).mul_add(6.0 * third, lower)
        } else if 2.0 * third < 1.0 {
            upper
        } else if 3.0 * third < 2.0 {
            (upper - lower).mul_add((2.0 / 3.0 - third) * 6.0, lower)
        } else {
            lower
        };

        *slot = channel(value);
    }

    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

/// Colour each printable character of `string`, starting the hue at `offset`.
fn rainbow(string: &str, offset: usize) -> String {
    if string.is_empty() {
        return string.to_owned();
    }

    let coloured = string
        .chars()
        .filter(|character| !is_ignored(*character))
        .count();
    if coloured == 0 {
        return string.to_owned();
    }

    #[expect(clippy::cast_precision_loss, reason = "the string is a demo banner")]
    let hue_step = 360.0 / coloured as f64;

    #[expect(clippy::cast_precision_loss, reason = "the offset is a frame counter")]
    let mut hue = (offset % 360) as f64;
    let mut characters = String::new();

    for character in string.chars() {
        if is_ignored(character) {
            characters.push(character);
        } else {
            characters.push_str(&chalk().hex(&hsl_to_hex(hue, 100.0, 50.0)).paint(character));
            hue = (hue + hue_step) % 360.0;
        }
    }

    characters
}

/// Redraw a single line in place, the way `log-update` does.
fn update_log(line: &str) {
    let mut stdout = io::stdout();
    // Carriage return, then erase the whole line, then write over it.
    let _ = write!(stdout, "\r\u{1b}[2K{line}");
    let _ = stdout.flush();
}

fn main() {
    println!();

    for index in 0..360 * 5 {
        update_log(&rainbow("We hope you enjoy Chalk! <3", index));
        thread::sleep(Duration::from_millis(2));
    }

    println!();
}
