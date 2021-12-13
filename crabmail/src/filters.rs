use mailparse::{parse_mail, MailHeaderMap, ParsedMail};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn time_ago(amount: &u64) -> askama::Result<String> {
    Ok(timeago(*amount))
}

const SOLAR_YEAR_SECS: u64 = 31556926;

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
    } else if diff < SOLAR_YEAR_SECS * 2 {
        amount = diff / (60 * 60 * 24);
        metric = "day";
    } else {
        amount = diff / SOLAR_YEAR_SECS * 2;
        metric = "year";
    }
    match amount {
        1 => format!("{} {} ago", amount, metric),
        _ => format!("{} {}s ago", amount, metric),
    }
}
