//! Prints one styled word per stream, so that a test can check that colour is
//! disabled when neither stream is a terminal.

use chalk::{chalk, chalk_stderr};

fn main() {
    println!(
        "{} {}",
        chalk().hex("#ff6159").paint("testout"),
        chalk_stderr().hex("#ff6159").paint("testerr")
    );
}
