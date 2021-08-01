use mailparse::{parse_mail, MailHeaderMap, ParsedMail};

pub fn get_header(email: &&ParsedMail, header: &str) -> askama::Result<String> {
    Ok(email
        .headers
        .get_first_value(header)
        .unwrap_or("".to_string()))
}

// NOTE this function is currently unsafe
pub fn get_body(email: &&ParsedMail) -> askama::Result<String> {
    let core_email = email.subparts.get(0).unwrap_or(email);

    #[cfg(feature = "html")]
    {
        use ammonia;
        use std::collections::HashSet;
        use std::iter::FromIterator;
        // TODO dont initialize each time
        // TODO sanitize id, classes, etc.
        let tags = HashSet::from_iter(vec!["a", "b", "i", "br", "p", "span", "u"]);
        if core_email.ctype.mimetype == "text/html" {
            let a = ammonia::Builder::new()
                .tags(tags)
                .clean(&core_email.get_body().unwrap_or("".to_string()))
                .to_string();
            return Ok(a);
        }
    }

    if core_email.ctype.mimetype == "text/plain" {
        // TODO html escape this.
        return Ok(core_email.get_body().unwrap_or("".to_string()));
    }
    return Ok(String::from("[No valid body found]"));
}

// pub fn get_attachment(email: &&ParsedMail) -> askama::Result<String> {}
