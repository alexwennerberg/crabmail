use mail_parser::DateTime;
use std::fs::{read, write};
use std::io::prelude::*;
use std::path::PathBuf;

// from https://github.com/protocolbuffers/upb/blob/22182e6e/upb/json_decode.c#L982-L992
// TODO make rust-idiomatic
pub fn epoch_days(y: u32, m: u32, d: u32) -> i64 {
    let year_base = 4800;
    let m_adj = match m < 3 {
        true => 0,
        false => m - 3,
    };
    let carry = match m_adj > m {
        true => 1,
        false => 0,
    };
    let adjust = carry * 12;
    let y_adj = m + year_base - carry;
    let month_days = ((m_adj + adjust) * 62719 + 769) / 2048;
    let leap_days = y_adj / 4 - y_adj / 100 + y_adj / 400;
    y_adj as i64 * 365 + leap_days as i64 + month_days as i64 + (d as i64 - 1) - 2472632
}

// leap seconds?
// TODO test
pub fn epoch_time(dt: &DateTime) -> i64 {
    let mut h = dt.hour as i64;
    let mut m = dt.minute as i64;
    let s = dt.second;
    let adj = match dt.tz_before_gmt {
        true => 1,
        false => -1,
    };
    h += dt.tz_hour as i64 * adj;
    m += dt.tz_minute as i64 * adj;

    return epoch_days(dt.year, dt.month, dt.day) * 86400 + h * 3600 + m * 60 + dt.second as i64;
}
