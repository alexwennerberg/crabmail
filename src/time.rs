/// Useful "Good enough" time utils in Rust
use std::time::{SystemTime, UNIX_EPOCH};

const DAYS_IN_MONTHS: [i64; 12] = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];

/* 2000-03-01 (mod 400 year, immediately after feb29 */
const LEAP_EPOCH: i64 = 946684800 + 86400 * (31 + 29);

const DAYS_PER_400Y: i64 = (365 * 400 + 97);
const DAYS_PER_100Y: i64 = (365 * 100 + 24);
const DAYS_PER_4Y: i64 = (365 * 4 + 1);

// TODO fiture out types
#[derive(Debug, Clone)]
pub struct Date {
    year: u32,
    month: u32,
    day_of_month: u32,
    day_of_week: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

impl Date {
    pub fn ymd(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day_of_month
        )
    }
}

// from http://git.musl-libc.org/cgit/musl/tree/src/time/__secs_to_tm.c
// with a slightly different API
// this is a line-for-line copy, not idiomatic rust
// UTC
// TODO handle 64-bit overflow
pub fn secs_to_date(unixtime: u64) -> Date {
    let secs = unixtime as i64 - LEAP_EPOCH;
    let mut days = secs / 86400;
    let mut remsecs = secs % 86400;
    if (remsecs < 0) {
        remsecs += 86400;
        days -= 1;
    }
    let mut wday = (3 + days) % 7;
    if (wday < 0) {
        wday += 7
    };
    let mut qc_cycles = days / DAYS_PER_400Y;
    let mut remdays = days % DAYS_PER_400Y;
    if (remdays < 0) {
        remdays += DAYS_PER_400Y;
        qc_cycles -= 1;
    }
    let mut c_cycles = remdays / DAYS_PER_100Y;
    if (c_cycles == 4) {
        c_cycles -= 1;
    }
    remdays -= c_cycles * DAYS_PER_100Y;

    let mut q_cycles = remdays / DAYS_PER_4Y;
    if (q_cycles == 25) {
        q_cycles -= 1
    }
    remdays -= q_cycles * DAYS_PER_4Y;

    let mut remyears = remdays / 365;
    if (remyears == 4) {
        remyears -= 1
    }
    remdays -= remyears * 365;

    // C
    let leap = match remyears == 0 && (q_cycles != 0 || c_cycles == 0) {
        true => 1,
        false => 0,
    };
    let mut yday = remdays + 31 + 28 + leap;
    if (yday >= 365 + leap) {
        yday -= 365 + leap
    }

    let mut years = remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;
    let mut months: i64 = 0;
    while DAYS_IN_MONTHS[months as usize] <= remdays {
        remdays -= DAYS_IN_MONTHS[months as usize];
        months += 1;
    }
    // some sort of weird off by 1 error from my musl translation
    months += 1;

    if (months > 10) {
        months -= 12;
        years += 1;
    }
    Date {
        year: (years + 2000) as u32,
        month: (months + 2) as u32,
        day_of_month: (remdays + 1) as u32,
        day_of_week: (wday) as u32,
        hour: (remsecs / 3600) as u32,
        minute: ((remsecs / 60) % 60) as u32,
        second: (remsecs % 60) as u32,
    }
}

const SOLAR_YEAR_SECS: u64 = 31556926;

pub fn timeago(unixtime: u64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_conversion() {
        assert_eq!("2021-12-22", secs_to_date(1640211435).ymd());
        assert_eq!("2022-01-04", secs_to_date(1641321435).ymd())
    }
}
