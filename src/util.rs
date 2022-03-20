// Truncate a string, adding ellpises if it's too long
fn truncate_ellipsis(s: &str, n: usize) -> String {
    let out = String::with_capacity(n);
    if s.len() <= n {
        return s[..n].to_owned();
    }
    let res = match s.char_indices().nth(n - 3) {
        None => s,
        Some((idx, _)) => &s[..idx],
    };
    out.push_str(res);
    res
}
