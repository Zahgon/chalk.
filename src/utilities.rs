//! String rewriting helpers used while applying a style.

use std::borrow::Cow;

/// Insert `postfix` after every occurrence of `substring`, keeping the match.
///
/// Note: each match is kept and `postfix` is inserted after it. `str::replace`
/// with `substring.to_owned() + postfix` does the same, but it always allocates
/// — including on the no-match path that most calls take. Returning a `Cow`
/// keeps that path free.
#[must_use]
pub fn string_replace_all<'a>(string: &'a str, substring: &str, postfix: &str) -> Cow<'a, str> {
    let Some(mut index) = string.find(substring) else {
        return Cow::Borrowed(string);
    };

    let substring_length = substring.len();
    let mut end_index = 0;
    let mut return_value = String::with_capacity(string.len() + postfix.len());

    loop {
        return_value.push_str(&string[end_index..index]);
        return_value.push_str(substring);
        return_value.push_str(postfix);
        end_index = index + substring_length;

        let Some(next) = string[end_index..].find(substring) else {
            break;
        };
        index = end_index + next;
    }

    return_value.push_str(&string[end_index..]);
    Cow::Owned(return_value)
}

/// Close a style before every line break and reopen it after.
///
/// `index` is the byte index of the first `\n`, which the caller has already
/// found. A `\r` immediately before a `\n` stays on the first line, ahead of
/// `prefix`, so that a CRLF pair is not split.
#[must_use]
pub fn string_encase_crlf_with_first_index(
    string: &str,
    prefix: &str,
    postfix: &str,
    index: usize,
) -> String {
    let bytes = string.as_bytes();
    let mut index = index;
    let mut end_index = 0;
    let mut return_value = String::with_capacity(string.len());

    loop {
        let is_got_cr = index > 0 && bytes[index - 1] == b'\r';

        return_value.push_str(&string[end_index..if is_got_cr { index - 1 } else { index }]);
        return_value.push_str(prefix);
        return_value.push_str(if is_got_cr { "\r\n" } else { "\n" });
        return_value.push_str(postfix);
        end_index = index + 1;

        let Some(next) = string[end_index..].find('\n') else {
            break;
        };
        index = end_index + next;
    }

    return_value.push_str(&string[end_index..]);
    return_value
}
