// Truncate a string, adding ellpises if it's too long
pub fn truncate_ellipsis(s: &str, n: usize) -> String {
    let mut out = String::with_capacity(n);
    if s.len() <= n {
        return s.to_owned();
    }
    let res = match s.char_indices().nth(n - 3) {
        None => s,
        Some((idx, _)) => &s[..idx],
    };
    out.push_str(res);
    out.push_str("...");
    out.to_string()
}

// TODO quoted lines?
// stolen from https://github.com/deltachat/deltachat-core-rust/blob/master/src/format_flowed.rs
// undoes format=flowed
pub fn unformat_flowed(text: &str) -> String {
    let mut result = String::new();
    let mut skip_newline = true;

    for line in text.split('\n') {
        // Revert space-stuffing
        let line = line.strip_prefix(' ').unwrap_or(line);

        if !skip_newline {
            result.push('\n');
        }

        if let Some(line) = line.strip_suffix(' ') {
            // Flowed line
            result += line;
            result.push(' ');
            skip_newline = true;
        } else {
            // Fixed line
            result += line;
            skip_newline = false;
        }
    }
    result
}

pub const EPOCH: mail_parser::DateTime = mail_parser::DateTime {
    year: 1970,
    month: 1,
    day: 1,
    hour: 0,
    minute: 0,
    second: 0,
    tz_before_gmt: false,
    tz_hour: 0,
    tz_minute: 0,
};
