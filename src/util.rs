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
