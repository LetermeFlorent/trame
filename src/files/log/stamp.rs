use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 3_600;
const SECONDS_PER_DAY: u64 = 86_400;
const DAYS_FROM_MARCH_ZERO_TO_EPOCH: u64 = 719_468;
const DAYS_PER_ERA: u64 = 146_097;
const DAYS_PER_CENTURY: u64 = 36_524;
const DAYS_PER_YEAR: u64 = 365;
const DAYS_FROM_MARCH_TO_JULY: u64 = 153;
const YEARS_PER_ERA: u64 = 400;
const MONTHS_PER_YEAR: u64 = 12;
const MARCH: u64 = 3;
const YEARS_PER_LEAP_CYCLE: u64 = 4;
const YEARS_PER_CENTURY: u64 = 100;
const MONTH_DAYS_SLOPE: u64 = 5;
const MONTH_DAYS_OFFSET: u64 = 2;

pub(super) fn stamp(at: SystemTime) -> String {
    let seconds = at
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let (year, month, day) = civil_date(seconds / SECONDS_PER_DAY);
    let clock = seconds % SECONDS_PER_DAY;
    let hour = clock / SECONDS_PER_HOUR;
    let minute = clock % SECONDS_PER_HOUR / SECONDS_PER_MINUTE;
    let second = clock % SECONDS_PER_MINUTE;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_date(days_since_epoch: u64) -> (u64, u64, u64) {
    let days = days_since_epoch + DAYS_FROM_MARCH_ZERO_TO_EPOCH;
    let era = days / DAYS_PER_ERA;
    let day_of_era = days % DAYS_PER_ERA;
    let leap_days = day_of_era / (YEARS_PER_LEAP_CYCLE * DAYS_PER_YEAR)
        - day_of_era / DAYS_PER_CENTURY
        + day_of_era / (DAYS_PER_ERA - 1);
    let year_of_era = (day_of_era - leap_days) / DAYS_PER_YEAR;
    let year_start = DAYS_PER_YEAR * year_of_era + year_of_era / YEARS_PER_LEAP_CYCLE
        - year_of_era / YEARS_PER_CENTURY;
    let day_of_year = day_of_era - year_start;
    let month_index =
        (MONTH_DAYS_SLOPE * day_of_year + MONTH_DAYS_OFFSET) / DAYS_FROM_MARCH_TO_JULY;
    let day = day_of_year
        - (DAYS_FROM_MARCH_TO_JULY * month_index + MONTH_DAYS_OFFSET) / MONTH_DAYS_SLOPE
        + 1;
    let month = (month_index + MARCH - 1) % MONTHS_PER_YEAR + 1;
    let year = era * YEARS_PER_ERA + year_of_era + u64::from(month < MARCH);
    (year, month, day)
}
