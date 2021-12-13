use mailparse::{parse_mail, MailHeaderMap, ParsedMail};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn time_ago(amount: &u64) -> askama::Result<String> {
    Ok(timeago(*amount))
}

fn timeago(unixtime: u64) -> String {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if unixtime > current_time {
        return "in the future".to_owned();
    }
    let diff = current_time - unixtime;
    let amount: u64;
    let metric: &str;
    if diff < 60 {
        amount = diff;
        metric = "second";
    } else if diff < 60 * 60 {
        amount = diff / 60;
        metric = "minute";
    } else if diff < 60 * 60 * 24 {
        amount = diff / (60 * 60);
        metric = "hour";
    } else {
        amount = diff / (60 * 60 * 24);
        metric = "day";
    }
    match amount {
        1 => format!("{} {} ago", amount, metric),
        _ => format!("{} {}s ago", amount, metric),
    }
}
// // NOTE this function is currently unsafe
// pub fn get_body(email: &&ParsedMail) -> askama::Result<String> {
//     let core_email = email.subparts.get(0).unwrap_or(email);

//     #[cfg(feature = "html")]
//     {
//         use ammonia;
//         use std::collections::HashSet;
//         use std::iter::FromIterator;
//         // TODO dont initialize each time
//         // TODO sanitize id, classes, etc.
//         let tags = HashSet::from_iter(vec!["a", "b", "i", "br", "p", "span", "u"]);
//         if core_email.ctype.mimetype == "text/html" {
//             let a = ammonia::Builder::new()
//                 .tags(tags)
//                 .clean(&core_email.get_body().unwrap_or("".to_string()))
//                 .to_string();
//             return Ok(a);
//         }
//     }

//     if core_email.ctype.mimetype == "text/plain" {
//         // TODO html escape this.
//         return Ok(core_email.get_body().unwrap_or("".to_string()));
//     }
//     return Ok(String::from("[No valid body found]"));
// }
