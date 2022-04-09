// less efficient, easier api
pub fn xml_safe(text: &str) -> String {
    // note we escape more than we need to
    let mut dest = Vec::new();
    xml_escape(text, &mut dest);
    std::str::from_utf8(&dest).unwrap().to_owned()
}

pub fn xml_escape(text: &str, dest: &mut Vec<u8>) {
    for c in text.bytes() {
        match c {
            b'&' => dest.extend_from_slice(b"&amp;"),
            b'<' => dest.extend_from_slice(b"&lt;"),
            b'>' => dest.extend_from_slice(b"&gt;"),
            b'"' => dest.extend_from_slice(b"&quot;"),
            b'\'' => dest.extend_from_slice(b"&#39;"),
            // Quick and dirty email obfuscation
            b'@' => dest.extend_from_slice(b"&#x40;"),
            _ => dest.push(c),
        }
    }
}
