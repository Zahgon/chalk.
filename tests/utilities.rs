//! Tests for the string rewriting helpers.
//!
//! The original covers these only through the styling tests, which reach every
//! line of `utilities.js` but only ever hand it one line break. The port splits
//! the loop differently, so the continuation path needs a case of its own.

use chalk::utilities::{string_encase_crlf_with_first_index, string_replace_all};

#[test]
fn string_replace_all_returns_the_input_untouched_when_there_is_no_match() {
    let replaced = string_replace_all("hello world", "\u{1b}[39m", "\u{1b}[31m");
    assert_eq!(replaced, "hello world");
    // The no-match path must not allocate.
    assert!(matches!(replaced, std::borrow::Cow::Borrowed(_)));

    assert_eq!(string_replace_all("", "x", "y"), "");
}

#[test]
fn string_replace_all_keeps_each_match_and_appends_the_postfix() {
    assert_eq!(string_replace_all("a-b", "-", "+"), "a-+b");
    assert_eq!(string_replace_all("a-b-c", "-", "+"), "a-+b-+c");
    assert_eq!(string_replace_all("-a-", "-", "+"), "-+a-+");
    assert_eq!(string_replace_all("--", "-", "+"), "-+-+");
    assert_eq!(
        string_replace_all("x\u{1b}[39my\u{1b}[39mz", "\u{1b}[39m", "\u{1b}[31m"),
        "x\u{1b}[39m\u{1b}[31my\u{1b}[39m\u{1b}[31mz"
    );
}

#[test]
fn string_replace_all_handles_overlapping_candidates_left_to_right() {
    // After a match the scan resumes past it, so `aa` in `aaa` matches once.
    assert_eq!(string_replace_all("aaa", "aa", "!"), "aa!a");
}

#[test]
fn string_encase_closes_and_reopens_around_every_line_break() {
    let encase = |string: &str| {
        let index = string.find('\n').expect("the caller has already found one");
        string_encase_crlf_with_first_index(string, "<close>", "<open>", index)
    };

    assert_eq!(encase("a\nb"), "a<close>\n<open>b");
    // More than one break exercises the loop's continuation.
    assert_eq!(encase("a\nb\nc"), "a<close>\n<open>b<close>\n<open>c");
    assert_eq!(encase("\n\n"), "<close>\n<open><close>\n<open>");
    assert_eq!(encase("a\n"), "a<close>\n<open>");
    assert_eq!(encase("\nb"), "<close>\n<open>b");
}

#[test]
fn string_encase_keeps_a_carriage_return_ahead_of_the_close() {
    let encase = |string: &str| {
        let index = string.find('\n').expect("the caller has already found one");
        string_encase_crlf_with_first_index(string, "<close>", "<open>", index)
    };

    assert_eq!(encase("a\r\nb"), "a<close>\r\n<open>b");
    assert_eq!(
        encase("a\r\nb\nc\r\nd"),
        "a<close>\r\n<open>b<close>\n<open>c<close>\r\n<open>d"
    );
    // A lone carriage return is not part of a pair and stays where it is.
    assert_eq!(encase("a\rb\nc"), "a\rb<close>\n<open>c");
}

#[test]
fn string_encase_does_not_split_a_multi_byte_character() {
    let string = "h\u{e9}llo \u{2192}\nw\u{f6}rld";
    let index = string.find('\n').unwrap();

    assert_eq!(
        string_encase_crlf_with_first_index(string, "<close>", "<open>", index),
        "h\u{e9}llo \u{2192}<close>\n<open>w\u{f6}rld"
    );
}
