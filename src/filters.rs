use mailparse::{parse_mail, MailHeaderMap, ParsedMail};

pub fn get_header(email: &&ParsedMail, header: &str) -> askama::Result<String> {
    Ok(email
        .headers
        .get_first_value(header)
        .unwrap_or("".to_string()))
}

pub fn get_body(email: &&ParsedMail) -> askama::Result<String> {
    Ok(email.get_body().unwrap_or("".to_string()))
}
