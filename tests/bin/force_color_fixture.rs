//! Prints the detected stdout colour level, for the `FORCE_COLOR` tests.

use chalk::supports_color;

fn main() {
    println!(
        "{}",
        supports_color().map_or(0, |support| support.level.as_u8())
    );
}
