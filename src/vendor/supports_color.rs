//! Detect whether the terminal supports colour.
//!
//! Port of the vendored `supports-color` module. The decision procedure in
//! [`detect_level`] is a direct transcription of the original's, including the
//! order of its branches, which is observable: several of them are
//! distinguished only by which one wins.

use std::collections::HashMap;
use std::io::IsTerminal as _;
use std::sync::LazyLock;

use crate::ColorSupportLevel;

/// How much colour a stream supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorSupport {
    /// The colour level.
    pub level: ColorSupportLevel,
    /// Whether basic 16 colours are supported.
    pub has_basic: bool,
    /// Whether ANSI 256 colours are supported.
    pub has_256: bool,
    /// Whether Truecolor 16 million colours are supported.
    pub has_16m: bool,
}

/// The detection result for a stream.
///
/// `None` means no colour support at all — the original returns the boolean
/// `false` in that case rather than a record with `level: 0`.
pub type ColorInfo = Option<ColorSupport>;

/// Options for [`create_supports_color`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Whether the process arguments should be sniffed for `--color` and
    /// `--no-color` flags.
    pub sniff_flags: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { sniff_flags: true }
    }
}

/// The process-global inputs the detection procedure reads.
///
/// The original reads `process.env`, `process.argv`, `process.platform`, and
/// `os.release()` directly at each decision point. Capturing them in one value
/// keeps the procedure a pure function of its inputs, which is what makes the
/// whole of it testable in-process rather than only through a spawned child.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inputs {
    /// The environment.
    pub env: HashMap<String, String>,
    /// The process arguments, including the program itself.
    pub args: Vec<String>,
    /// The operating-system release string, on Windows only.
    ///
    /// `None` everywhere else, which skips the Windows branch entirely — the
    /// original guards that branch with `process.platform === 'win32'`.
    pub windows_release: Option<String>,
}

impl Inputs {
    /// Capture the current process's environment, arguments, and OS release.
    #[must_use]
    pub fn from_process() -> Self {
        Self {
            env: std::env::vars().collect(),
            args: std::env::args().collect(),
            windows_release: windows_release(),
        }
    }

    /// Whether a variable is present in the environment, however it is spelled.
    fn has(&self, key: &str) -> bool {
        self.env.contains_key(key)
    }

    /// The value of an environment variable, or `""` when it is absent.
    ///
    /// The original reads `env.X` and hands the result — possibly `undefined` —
    /// straight to a regular expression or an equality test. For every variable
    /// read that way, `undefined` and `""` lead to the same branch, so an
    /// absent variable is an empty string here.
    fn get(&self, key: &str) -> &str {
        self.env.get(key).map_or("", String::as_str)
    }

    /// Whether a flag appears in the arguments before any `--` terminator.
    ///
    /// A flag that already starts with a dash is matched verbatim, a
    /// single-character one takes `-`, and anything else takes `--`.
    ///
    /// From <https://github.com/sindresorhus/has-flag/blob/main/index.js>
    #[must_use]
    pub fn has_flag(&self, flag: &str) -> bool {
        let prefix = if flag.starts_with('-') {
            ""
        } else if flag.chars().count() == 1 {
            "-"
        } else {
            "--"
        };

        let needle = format!("{prefix}{flag}");
        let position = self.args.iter().position(|argument| *argument == needle);
        let terminator = self.args.iter().position(|argument| argument == "--");

        match (position, terminator) {
            (None, _) => false,
            (Some(_), None) => true,
            (Some(found), Some(terminator)) => found < terminator,
        }
    }
}

/// The OS release string, on Windows.
///
/// This is the crate's only `unsafe`. The crate-level `unsafe_code` lint is
/// silenced here rather than crate-wide, so that any *other* use of `unsafe`
/// still has to be argued for.
#[cfg(windows)]
#[expect(
    unsafe_code,
    reason = "reading the OS version needs an FFI call; there is no std equivalent"
)]
fn windows_release() -> Option<String> {
    /// `RTL_OSVERSIONINFOW`, as `RtlGetVersion` fills it in.
    #[repr(C)]
    struct OsVersionInfoW {
        os_version_info_size: u32,
        major_version: u32,
        minor_version: u32,
        build_number: u32,
        platform_id: u32,
        csd_version: [u16; 128],
    }

    // Node's `os.release()` reads the version through `RtlGetVersion` rather
    // than `GetVersionEx`, because the latter reports a capped version for a
    // process without a compatibility manifest.
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn RtlGetVersion(version_information: *mut OsVersionInfoW) -> i32;
    }

    let mut info = OsVersionInfoW {
        os_version_info_size: u32::try_from(size_of::<OsVersionInfoW>()).ok()?,
        major_version: 0,
        minor_version: 0,
        build_number: 0,
        platform_id: 0,
        csd_version: [0; 128],
    };

    // SAFETY: `info` is a correctly sized, fully initialised `RTL_OSVERSIONINFOW`.
    let status = unsafe { RtlGetVersion(&raw mut info) };
    if status != 0 {
        return None;
    }

    Some(format!(
        "{}.{}.{}",
        info.major_version, info.minor_version, info.build_number
    ))
}

/// The OS release string — never consulted anywhere but Windows.
#[cfg(not(windows))]
fn windows_release() -> Option<String> {
    None
}

/// `Number(string)`, restricted to what the callers can produce.
///
/// Returns `None` where JavaScript would produce `NaN`, so that a comparison
/// against it is false — which is how the original's `Number(osRelease[0]) >= 10`
/// behaves for a release string it cannot parse.
fn js_number(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Some(0.0);
    }

    trimmed.parse().ok()
}

/// `Number.parseInt(value, 10)`, which reads the *leading* integer of a string.
///
/// Skips leading whitespace and an optional sign, takes the run of ASCII digits
/// that follows, and ignores whatever trails it — so `"3abc"` is `3`, which is
/// how the original reads a `TERM_PROGRAM_VERSION` of `"3abc"` as major version
/// 3. A run too long for `i64` saturates rather than failing, because the only
/// use compares against 3. Returns `None` where JavaScript would produce `NaN`.
fn js_parse_int(value: &str) -> Option<i64> {
    let trimmed = value.trim_start();
    let (negative, digits) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };

    let end = digits
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(digits.len());
    if end == 0 {
        return None;
    }

    let magnitude = digits[..end].parse::<i64>().unwrap_or(i64::MAX);
    Some(if negative { -magnitude } else { magnitude })
}

/// The colour level a Windows release string implies.
///
/// Windows 10 build 10586 is the first Windows release that supports 256
/// colours. Windows 10 build 14931 is the first release that supports
/// 16m/Truecolor.
fn windows_color_level(release: &str) -> u8 {
    let parts: Vec<&str> = release.split('.').collect();
    let major = parts.first().copied().and_then(js_number);
    let build = parts.get(2).copied().and_then(js_number);

    if let (Some(major), Some(build)) = (major, build)
        && major >= 10.0
        && build >= 10_586.0
    {
        return if build >= 14_931.0 { 3 } else { 2 };
    }

    1
}

/// Whether `FORCE_COLOR` names a level.
///
/// Shared with the exact-level check in [`detect_level`] so that the two cannot
/// disagree on what counts as numeric. Mirrors `/^\d+$/`.
fn has_numeric_force_color(inputs: &Inputs) -> bool {
    let value = inputs.get("FORCE_COLOR");
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

/// How `FORCE_COLOR` forces the level, if it does.
fn env_force_color(inputs: &Inputs) -> Option<u8> {
    if !inputs.has("FORCE_COLOR") {
        return None;
    }

    let value = inputs.get("FORCE_COLOR");

    if value == "false" {
        return Some(0);
    }

    if value == "true" || value.is_empty() {
        return Some(1);
    }

    if !has_numeric_force_color(inputs) {
        return None;
    }

    // A value too long to fit an integer is still numeric, and still clamps.
    Some(
        value
            .parse::<u64>()
            .unwrap_or(u64::MAX)
            .min(3)
            .try_into()
            .unwrap_or(3),
    )
}

/// How the `--color` family of flags forces the level, if it does.
fn flag_force_color(inputs: &Inputs) -> Option<u8> {
    if inputs.has_flag("no-color")
        || inputs.has_flag("no-colors")
        || inputs.has_flag("color=false")
        || inputs.has_flag("color=never")
    {
        return Some(0);
    }

    if inputs.has_flag("color")
        || inputs.has_flag("colors")
        || inputs.has_flag("color=true")
        || inputs.has_flag("color=always")
    {
        return Some(1);
    }

    None
}

/// Whether the `TEAMCITY_VERSION` value is new enough to support colour.
///
/// Mirrors `/^(?:9\.0*[1-9]\d*\.|\d{2,}\.)/`: either `9.` followed by a
/// non-zero build number and a `.`, or a two-or-more digit major version and a
/// `.`.
fn teamcity_supports_color(version: &str) -> bool {
    /// The length of the run of ASCII digits at the start of `value`.
    fn leading_digits(value: &str) -> usize {
        value
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(value.len())
    }

    if let Some(rest) = version.strip_prefix("9.") {
        // `0*` is greedy, and backtracking into it could only offer a `0` to
        // `[1-9]`, which cannot match — so the greedy split is the only one.
        let rest = rest.trim_start_matches('0');
        if rest.starts_with(|character: char| character.is_ascii_digit() && character != '0') {
            let tail = &rest[leading_digits(rest)..];
            if tail.starts_with('.') {
                return true;
            }
        }
    }

    let digits = leading_digits(version);
    digits >= 2 && version[digits..].starts_with('.')
}

/// Whether `TERM` matches `/-256(?:color)?$/i`.
fn term_is_256(term: &str) -> bool {
    let term = term.to_ascii_lowercase();
    term.ends_with("-256") || term.ends_with("-256color")
}

/// Whether `TERM` matches
/// `/^screen|^xterm|^vt100|^vt220|^rxvt|color|ansi|cygwin|linux/i`.
fn term_is_basic(term: &str) -> bool {
    let term = term.to_ascii_lowercase();

    term.starts_with("screen")
        || term.starts_with("xterm")
        || term.starts_with("vt100")
        || term.starts_with("vt220")
        || term.starts_with("rxvt")
        || term.contains("color")
        || term.contains("ansi")
        || term.contains("cygwin")
        || term.contains("linux")
}

/// The colour level for a stream, as a number from 0 to 3.
///
/// `stream_is_tty` is `None` when there is no stream at all — which, as in the
/// original, skips the not-a-TTY branch rather than taking it.
///
/// The branch order below is the original's and must not be rearranged.
#[must_use]
pub fn detect_level(inputs: &Inputs, stream_is_tty: Option<bool>, sniff_flags: bool) -> u8 {
    let env_forced = env_force_color(inputs);

    // The original assigns a defined `envForceColor()` over the module-level
    // flag value, then reads the flag value back; that is this `or`.
    let force_color = if sniff_flags {
        env_forced.or_else(|| flag_force_color(inputs))
    } else {
        env_forced
    };

    if force_color == Some(0) {
        return 0;
    }

    if sniff_flags {
        if inputs.has_flag("color=16m")
            || inputs.has_flag("color=full")
            || inputs.has_flag("color=truecolor")
        {
            return 3;
        }

        if inputs.has_flag("color=256") {
            return 2;
        }
    }

    // A numeric `FORCE_COLOR` requests an exact level, while `FORCE_COLOR=true`
    // and `FORCE_COLOR=` only enable colour and let the level be detected.
    if let Some(force_color) = force_color
        && has_numeric_force_color(inputs)
    {
        return force_color;
    }

    // Check for Azure DevOps pipelines.
    // Has to be above the `!streamIsTTY` check.
    if inputs.has("TF_BUILD") && inputs.has("AGENT_NAME") {
        return 1;
    }

    if stream_is_tty == Some(false) && force_color.is_none() {
        return 0;
    }

    let min = force_color.unwrap_or(0);

    if inputs.get("TERM") == "dumb" {
        return min;
    }

    if let Some(release) = &inputs.windows_release {
        return windows_color_level(release);
    }

    if inputs.has("CI") {
        if ["GITHUB_ACTIONS", "GITEA_ACTIONS", "CIRCLECI"]
            .iter()
            .any(|key| inputs.has(key))
        {
            return 3;
        }

        if ["TRAVIS", "APPVEYOR", "GITLAB_CI", "BUILDKITE", "DRONE"]
            .iter()
            .any(|sign| inputs.has(sign))
            || inputs.get("CI_NAME") == "codeship"
        {
            return 1;
        }

        return min;
    }

    if inputs.has("TEAMCITY_VERSION") {
        return u8::from(teamcity_supports_color(inputs.get("TEAMCITY_VERSION")));
    }

    if inputs.get("COLORTERM") == "truecolor" {
        return 3;
    }

    if inputs.get("TERM") == "xterm-kitty" {
        return 3;
    }

    if inputs.get("TERM") == "xterm-ghostty" {
        return 3;
    }

    if inputs.get("TERM") == "wezterm" {
        return 3;
    }

    if inputs.has("TERM_PROGRAM") {
        let version = js_parse_int(
            inputs
                .get("TERM_PROGRAM_VERSION")
                .split('.')
                .next()
                .unwrap_or(""),
        );

        match inputs.get("TERM_PROGRAM") {
            "iTerm.app" => return if version >= Some(3) { 3 } else { 2 },
            "Apple_Terminal" => return 2,
            _ => {}
        }
    }

    if term_is_256(inputs.get("TERM")) {
        return 2;
    }

    if term_is_basic(inputs.get("TERM")) {
        return 1;
    }

    if inputs.has("COLORTERM") {
        return 1;
    }

    min
}

/// Turn a numeric level into the reported shape.
fn translate_level(level: u8) -> ColorInfo {
    if level == 0 {
        return None;
    }

    Some(ColorSupport {
        level: ColorSupportLevel::from_detected(level),
        has_basic: true,
        has_256: level >= 2,
        has_16m: level >= 3,
    })
}

/// Detect the colour support of a stream.
///
/// `stream_is_tty` is `None` when the caller has no stream in hand.
#[must_use]
pub fn create_supports_color(stream_is_tty: Option<bool>, options: Options) -> ColorInfo {
    create_supports_color_from(&Inputs::from_process(), stream_is_tty, options)
}

/// Detect the colour support implied by an explicit set of inputs.
#[must_use]
pub fn create_supports_color_from(
    inputs: &Inputs,
    stream_is_tty: Option<bool>,
    options: Options,
) -> ColorInfo {
    translate_level(detect_level(inputs, stream_is_tty, options.sniff_flags))
}

/// The colour support of this process's standard output and standard error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupportsColor {
    /// The colour support of standard output.
    pub stdout: ColorInfo,
    /// The colour support of standard error.
    pub stderr: ColorInfo,
}

/// The colour support detected for this process, computed once.
///
/// The original computes this at module load; a `LazyLock` computes it at first
/// use, which is the closest equivalent that does not run before `main`.
pub static SUPPORTS_COLOR: LazyLock<SupportsColor> = LazyLock::new(|| {
    let inputs = Inputs::from_process();
    let options = Options::default();

    SupportsColor {
        stdout: create_supports_color_from(&inputs, Some(std::io::stdout().is_terminal()), options),
        stderr: create_supports_color_from(&inputs, Some(std::io::stderr().is_terminal()), options),
    }
});
