<h1 align="center">
	<br>
	<br>
	<img width="320" src="media/logo.svg" alt="Chalk">
	<br>
	<br>
	<br>
</h1>

> Terminal string styling done right

[![Coverage Status](https://codecov.io/gh/chalk/chalk/branch/main/graph/badge.svg)](https://codecov.io/gh/chalk/chalk)

![](media/screenshot.png)

## Info

- [Why not switch to a smaller coloring package?](https://github.com/chalk/chalk?tab=readme-ov-file#why-not-switch-to-a-smaller-coloring-package)
- See [yoctocolors](https://github.com/sindresorhus/yoctocolors) for a smaller alternative

## Highlights

- Expressive API
- Highly performant
- No dependencies
- Ability to nest styles
- [256/Truecolor color support](#256-and-truecolor-color-support)
- Auto-detects color support
- Clean and focused
- Actively maintained

## Install

```sh
cargo add chalk
```

## Usage

```rust
use chalk::chalk;

println!("{}", chalk().blue().paint("Hello world!"));
```

Chalk comes with an easy to use composable API where you just chain and nest the styles you want.

```rust
use chalk::chalk;

let chalk = chalk();

// Combine styled and normal strings
println!("{} World{}", chalk.blue().paint("Hello"), chalk.red().paint("!"));

// Compose multiple styles using the chainable API
println!("{}", chalk.blue().bg_red().bold().paint("Hello world!"));

// Pass in multiple arguments
println!("{}", chalk.blue().paint_all(["Hello", "World!", "Foo", "bar", "biz", "baz"]));

// Nest styles
println!("{}", chalk.red().paint(format!(
	"Hello {}!",
	chalk.underline().bg_blue().paint("world"),
)));

// Nest styles of the same type even (color, underline, background)
println!("{}", chalk.green().paint(format!(
	"I am a green line {} that becomes green again!",
	chalk.blue().underline().bold().paint("with a blue substring"),
)));

// Interpolation
println!("
CPU: {}
RAM: {}
DISK: {}
",
	chalk.red().paint("90%"),
	chalk.green().paint("40%"),
	chalk.yellow().paint("70%"),
);

// Use RGB colors in terminal emulators that support it.
println!("{}", chalk.rgb(123, 45, 67).underline().paint("Underlined reddish color"));
println!("{}", chalk.hex("#DEADED").bold().paint("Bold gray!"));
```

Easily define your own themes:

```rust
use chalk::chalk;

let chalk = chalk();

let error = chalk.bold().red();
let warning = chalk.hex("#FFA500"); // Orange color

println!("{}", error.paint("Error!"));
println!("{}", warning.paint("Warning!"));
```

## API

### `chalk.<style>()[.<style>()...].paint(value)`

Example: `chalk.red().bold().underline().paint("Hello world");`

Chain [styles](#styles) and call `paint` on the last one. Order doesn't matter, and later styles take precedent in case of a conflict. This simply means that `chalk.red().yellow().green()` is equivalent to `chalk.green()`.

`paint` takes anything that implements [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html), so it stringifies the value the same way `{}` would.

`paint_all` takes an iterator of such values and separates them by a space:

```rust
use chalk::chalk;

println!("{}", chalk().red().paint_all(["Hello", "world"]));
```

Calling `paint` on the instance itself applies no styling at all — it only stringifies, and `paint_all` only joins.

### `chalk.level()` / `chalk.set_level(level)`

Specifies the level of color support.

Color support is automatically detected, but you can override it by setting the level. You should however only do this in your own code as `chalk()` is shared by all Chalk consumers in the process.

If you need to change this in a reusable module, create a new instance:

```rust
use chalk::{Chalk, ColorSupportLevel};

let custom_chalk = Chalk::with_level(ColorSupportLevel::None);
```

| Level | `ColorSupportLevel` | Description |
| :---: | :--- | :--- |
| `0` | `None` | All colors disabled |
| `1` | `Basic` | Basic color support (16 colors) |
| `2` | `Ansi256` | 256 color support |
| `3` | `TrueColor` | Truecolor support (16 million colors) |

`ColorSupportLevel` makes an out-of-range level unrepresentable, so `set_level` cannot fail. Where the level comes from somewhere dynamic — a flag, a config file — `Chalk::try_with_level` and `try_set_level` take an `i64`, an `f64`, or a parsed string and return `Err(InvalidLevel)` for anything that is not an integer from 0 to 3. `Chalk::new`, and `Chalk::with_options` with `level: None`, have the level detected instead.

Every style obtained from an instance reads and writes *that instance's* level, however deep the chain, so a style held in a binding keeps following the level it was made from.

### `supports_color()`

Detect whether the terminal [supports color](https://github.com/chalk/supports-color). Used internally and handled for you, but exposed for convenience.

Can be overridden by the user with the flags `--color` and `--no-color`. For situations where using `--color` is not possible, use the environment variable `FORCE_COLOR=1` (level 1), `FORCE_COLOR=2` (level 2), or `FORCE_COLOR=3` (level 3) to forcefully enable color, or `FORCE_COLOR=0` to forcefully disable. A numeric `FORCE_COLOR` overrides the detected color support and sets the level directly, meaning the terminal cannot raise it to a higher level. Use `FORCE_COLOR=true` to instead only enable color and let the level be detected.

Explicit 256/Truecolor mode can be enabled using the `--color=256` and `--color=16m` flags, respectively. These take precedence over a non-zero numeric `FORCE_COLOR`.

### `chalk_stderr()` and `supports_color_stderr()`

`chalk_stderr()` returns a separate instance configured with color support detected for the `stderr` stream instead of `stdout`. Override rules from `supports_color` apply to this too. `supports_color_stderr` is exposed for convenience.

### `MODIFIER_NAMES`, `FOREGROUND_COLOR_NAMES`, `BACKGROUND_COLOR_NAMES`, `UNDERLINE_COLOR_NAMES`, and `COLOR_NAMES`

All supported style strings are exposed as a slice of strings for convenience. `COLOR_NAMES` is the combination of `FOREGROUND_COLOR_NAMES` and `BACKGROUND_COLOR_NAMES`. Underline color names are kept separate in `UNDERLINE_COLOR_NAMES`.

These names are data rather than Rust identifiers, so they are spelled the way the styles have always been spelled — `underlineCurly`, not `underline_curly`. Pass one to `style_by_name` to look the style up at runtime.

This can be useful if you wrap Chalk and need to validate input:

```rust
use chalk::{FOREGROUND_COLOR_NAMES, MODIFIER_NAMES};

println!("{}", MODIFIER_NAMES.contains(&"bold"));
//=> true

println!("{}", FOREGROUND_COLOR_NAMES.contains(&"pink"));
//=> false
```

## Styles

Every style below is a method spelled in snake case: `underlineCurly` is `underline_curly()`, `bgBlackBright` is `bg_black_bright()`.

### Modifiers

- `reset` - Reset the current style.
- `bold` - Make the text bold.
- `dim` - Make the text have lower opacity.
- `italic` - Make the text italic. *(Not widely supported)*
- `underline` - Put a horizontal line below the text. *(Not widely supported)*
- `underlineDouble` - Put a double horizontal line below the text. *(Not widely supported)*
- `underlineCurly` - Put a curly horizontal line below the text. *(Not widely supported)*
- `underlineDotted` - Put a dotted horizontal line below the text. *(Not widely supported)*
- `underlineDashed` - Put a dashed horizontal line below the text. *(Not widely supported)*
- `overline` - Put a horizontal line above the text. *(Not widely supported)*
- `inverse` - Invert background and foreground colors.
- `hidden` - Print the text but make it invisible.
- `strikethrough` - Puts a horizontal line through the center of the text. *(Not widely supported)*
- `visible` - Print the text only when Chalk has a color level above zero. Can be useful for things that are purely cosmetic.

### Colors

- `black`
- `red`
- `green`
- `yellow`
- `blue`
- `magenta`
- `cyan`
- `white`
- `blackBright` (alias: `gray`, `grey`)
- `redBright`
- `greenBright`
- `yellowBright`
- `blueBright`
- `magentaBright`
- `cyanBright`
- `whiteBright`

### Background colors

- `bgBlack`
- `bgRed`
- `bgGreen`
- `bgYellow`
- `bgBlue`
- `bgMagenta`
- `bgCyan`
- `bgWhite`
- `bgBlackBright` (alias: `bgGray`, `bgGrey`)
- `bgRedBright`
- `bgGreenBright`
- `bgYellowBright`
- `bgBlueBright`
- `bgMagentaBright`
- `bgCyanBright`
- `bgWhiteBright`

### Underline colors

The underline color is set independently of the text color, so the color is only visible when an underline style is also applied. For example, `chalk.underline_red().underline_curly().paint("typo")` renders a red squiggle below otherwise unstyled text. *(Not widely supported)*

Unlike text and background colors, there is no basic 16-color form for underline colors, so they always use the 256-color escape. At level 1 they are downsampled to the first 16 palette entries rather than to a basic color code.

- `underlineBlack`
- `underlineRed`
- `underlineGreen`
- `underlineYellow`
- `underlineBlue`
- `underlineMagenta`
- `underlineCyan`
- `underlineWhite`
- `underlineBlackBright` (alias: `underlineGray`, `underlineGrey`)
- `underlineRedBright`
- `underlineGreenBright`
- `underlineYellowBright`
- `underlineBlueBright`
- `underlineMagentaBright`
- `underlineCyanBright`
- `underlineWhiteBright`

## 256 and Truecolor color support

Chalk supports 256 colors and [Truecolor](https://github.com/termstandard/colors) (16 million colors) on supported terminal apps.

Colors are downsampled from 16 million RGB values to an ANSI color format that is supported by the terminal emulator (or by specifying a level as a Chalk option). For example, Chalk configured to run at level 1 (basic color support) will downsample an RGB value of #FF0000 (red) to 91 (ANSI escape for bright red). The same applies to `ansi256` values, so `chalk.ansi256(196)` also becomes 91 at level 1.

Examples:

- `chalk.hex("#DEADED").underline().paint("Hello, world!")`
- `chalk.rgb(15, 100, 204).inverse().paint("Hello!")`

Background versions of these models are prefixed with `bg_` (e.g. `hex` for foreground colors and `bg_hex` for background colors).

- `chalk.bg_hex("#DEADED").underline().paint("Hello, world!")`
- `chalk.bg_rgb(15, 100, 204).inverse().paint("Hello!")`

Underline versions are prefixed with `underline_` in the same way (e.g. `hex` for foreground colors and `underline_hex` for underline colors). They only take effect when an underline style is also applied.

- `chalk.underline_hex("#DEADED").underline_curly().paint("Hello, world!")`
- `chalk.underline_rgb(15, 100, 204).underline().paint("Hello!")`

The following color models can be used:

- [`rgb`](https://en.wikipedia.org/wiki/RGB_color_model) - Example: `chalk.rgb(255, 136, 0).bold().paint("Orange!")`
- [`hex`](https://en.wikipedia.org/wiki/Web_colors#Hex_triplet) - Example: `chalk.hex("#FF8800").bold().paint("Orange!")`
- [`ansi256`](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit) - Example: `chalk.bg_ansi256(194).paint("Honeydew, more or less")`

## Browser support

The JavaScript package ships a browser build that detects Chrome's native support for ANSI escape codes in the developer console. That build has no counterpart here: this crate detects a terminal, not a browser.

## Windows

If you're on Windows, do yourself a favor and use [Windows Terminal](https://github.com/microsoft/terminal) instead of `cmd.exe`.

## Development

```sh
cargo test                # run the test suite
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo bench               # run the benchmarks
cargo run --example rainbow
```

## FAQ

### Why not switch to a smaller coloring package?

Chalk may be larger, but there is a reason for that. It offers a more user-friendly API, well-documented types, supports millions of colors, and covers edge cases that smaller alternatives miss. Chalk is mature, reliable, and built to last.

But beyond the technical aspects, there's something more critical: trust and long-term maintenance. I have been active in open source for over a decade, and I'm committed to keeping Chalk maintained. Smaller packages might seem appealing now, but there's no guarantee they will be around for the long term, or that they won't become malicious over time.

If the goal is to clean up the ecosystem, switching away from Chalk won’t even make a dent. The real problem lies with packages that have very deep dependency trees (for example, those including a lot of polyfills). Chalk has no dependencies. It's better to focus on impactful changes rather than minor optimizations.

If absolute package size is important to you, I also maintain [yoctocolors](https://github.com/sindresorhus/yoctocolors), one of the smallest color packages out there.

*\- [Sindre](https://github.com/sindresorhus)*

### But the smaller coloring package has benchmarks showing it is faster

[Micro-benchmarks are flawed](https://sindresorhus.com/blog/micro-benchmark-fallacy) because they measure performance in unrealistic, isolated scenarios, often giving a distorted view of real-world performance. Don't believe marketing fluff. All the coloring packages are more than fast enough.

## Related

- [chalk-template](https://github.com/chalk/chalk-template) - [Tagged template literals](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals#tagged_templates) support for this module
- [chalk-cli](https://github.com/chalk/chalk-cli) - CLI for this module
- [ansi-styles](https://github.com/chalk/ansi-styles) - ANSI escape codes for styling strings in the terminal
- [supports-color](https://github.com/chalk/supports-color) - Detect whether a terminal supports color
- [strip-ansi](https://github.com/chalk/strip-ansi) - Strip ANSI escape codes
- [strip-ansi-stream](https://github.com/chalk/strip-ansi-stream) - Strip ANSI escape codes from a stream
- [has-ansi](https://github.com/chalk/has-ansi) - Check if a string has ANSI escape codes
- [ansi-regex](https://github.com/chalk/ansi-regex) - Regular expression for matching ANSI escape codes
- [wrap-ansi](https://github.com/chalk/wrap-ansi) - Wordwrap a string with ANSI escape codes
- [slice-ansi](https://github.com/chalk/slice-ansi) - Slice a string with ANSI escape codes
- [color-convert](https://github.com/qix-/color-convert) - Converts colors between different models
- [chalk-animation](https://github.com/bokub/chalk-animation) - Animate strings in the terminal
- [gradient-string](https://github.com/bokub/gradient-string) - Apply color gradients to strings
- [chalk-pipe](https://github.com/LitoMore/chalk-pipe) - Create chalk style schemes with simpler style strings
- [terminal-link](https://github.com/sindresorhus/terminal-link) - Create clickable links in the terminal

*(Not accepting additional entries)*

## Maintainers

- [Sindre Sorhus](https://github.com/sindresorhus)
- [Josh Junon](https://github.com/qix-)
