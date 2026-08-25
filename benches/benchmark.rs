//! Timing for the paths the library is expected to be hot on.
//!
//! Run with `cargo bench`. Deliberately harness-free: the crate has no
//! dependencies and a benchmark runner would be the first one.

use std::hint::black_box;
use std::time::{Duration, Instant};

use chalk::{Chalk, ColorSupportLevel, Style};

/// How long each case is run for before its per-iteration cost is reported.
const MEASURE_FOR: Duration = Duration::from_millis(500);

/// Time one case and print its throughput.
fn bench(name: &str, mut body: impl FnMut()) {
    // Warm up, so that the first timed iteration is not the one that allocates
    // the lazily-built statics.
    for _ in 0..1_000 {
        body();
    }

    let start = Instant::now();
    let mut iterations: u64 = 0;

    while start.elapsed() < MEASURE_FOR {
        for _ in 0..1_000 {
            body();
        }
        iterations += 1_000;
    }

    let elapsed = start.elapsed();
    #[expect(
        clippy::cast_precision_loss,
        reason = "an approximate rate is all a benchmark report needs"
    )]
    let per_second = iterations as f64 / elapsed.as_secs_f64();
    let nanos = elapsed.as_nanos() / u128::from(iterations);

    println!("  {name:<40} {per_second:>12.0} ops/sec  ({nanos} ns/op)");
}

fn main() {
    let chalk = Chalk::with_level(ColorSupportLevel::TrueColor);

    let chalk_red: Style = chalk.red();
    let chalk_bg_red: Style = chalk.bg_red();
    let chalk_blue_bg_red: Style = chalk.blue().bg_red();
    let chalk_blue_bg_red_bold: Style = chalk.blue().bg_red().bold();

    let blue_styled_string = format!("the fox jumps{}!", chalk.blue().paint("over the lazy dog"));

    println!("chalk");

    bench("1 style", || {
        black_box(chalk.red().paint("the fox jumps over the lazy dog"));
    });

    bench("2 styles", || {
        black_box(
            chalk
                .blue()
                .bg_red()
                .paint("the fox jumps over the lazy dog"),
        );
    });

    bench("3 styles", || {
        black_box(
            chalk
                .blue()
                .bg_red()
                .bold()
                .paint("the fox jumps over the lazy dog"),
        );
    });

    bench("cached: 1 style", || {
        black_box(chalk_red.paint("the fox jumps over the lazy dog"));
    });

    bench("cached: 2 styles", || {
        black_box(chalk_blue_bg_red.paint("the fox jumps over the lazy dog"));
    });

    bench("cached: 3 styles", || {
        black_box(chalk_blue_bg_red_bold.paint("the fox jumps over the lazy dog"));
    });

    bench("cached: 1 style with newline", || {
        black_box(chalk_red.paint("the fox jumps\nover the lazy dog"));
    });

    bench("cached: 1 style nested intersecting", || {
        black_box(chalk_red.paint(&blue_styled_string));
    });

    bench("cached: 1 style nested non-intersecting", || {
        black_box(chalk_bg_red.paint(&blue_styled_string));
    });
}
