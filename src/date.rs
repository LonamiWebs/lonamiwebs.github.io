use std::time::{SystemTime, UNIX_EPOCH};

static NON_LEAP_DAYS_PER_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
static LEAP_DAYS_PER_MONTH: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

#[inline]
fn is_leap_year(year: u16) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[inline]
fn days_in_year(year: u16) -> u16 {
    if is_leap_year(year) { 366 } else { 365 }
}

#[inline]
fn days_in_year_month(year: u16, month: u8) -> u8 {
    let days_in_month = if is_leap_year(year) {
        LEAP_DAYS_PER_MONTH
    } else {
        NON_LEAP_DAYS_PER_MONTH
    };
    days_in_month[month as usize]
}

pub fn system_time_to_date_string(time: SystemTime) -> String {
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .expect("time to be after unix epoch")
        .as_secs();

    let mut days = seconds / (24 * 60 * 60);

    let mut year = 1970;
    loop {
        let delta = u64::from(days_in_year(year));
        if days < delta {
            break;
        }
        days -= delta;
        year += 1;
    }

    let mut month = 0;
    loop {
        let delta = u64::from(days_in_year_month(year, month));
        if days < delta {
            break;
        }
        days -= delta;
        month = (month + 1) % 12;
    }

    month += 1;
    days += 1;
    format!("{year:04}-{month:02}-{days:02}")
}
