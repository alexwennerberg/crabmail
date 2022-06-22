// less efficient, easier api
pub fn xml_safe(text: &str) -> String {
    // note we escape more than we need to
    text.chars()
        .map(|c| {
            match c {
                '&' => "&amp;",
                '<' => "&lt;",
                '>' => "&gt;",
                '"' => "&quot;",
                '\'' => "&#39;",
                // Quick and dirty email obfuscation
                '@' => "&#x40;",
                // need an explicit return here because c is not a &str
                _ => return c.to_string(),
            }
            .to_string()
        })
        .collect::<String>()
}
