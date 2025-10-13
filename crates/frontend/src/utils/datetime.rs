use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::{Europe, Tz};

const DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

const TIME_ZONE: Tz = Europe::Stockholm;

pub fn datetime_to_input(datetime: &DateTime<Utc>) -> String {
    datetime
        .with_timezone(&TIME_ZONE)
        .naive_local()
        .format(DATE_FORMAT)
        .to_string()
}

pub fn input_to_datetime(input: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(input, DATE_FORMAT)
        .ok()
        .and_then(|dt| dt.and_local_timezone(TIME_ZONE).single())
        .map(|dt| dt.to_utc())
}

pub fn fmt_datetime(datetime: &DateTime<Utc>) -> String {
    datetime
        .with_timezone(&TIME_ZONE)
        .format("%a, %d %b %Y %H:%M:%S")
        .to_string()
}

pub fn fmt_datetime_opt(datetime: Option<&DateTime<Utc>>, none: &'static str) -> String {
    match datetime {
        Some(dt) => fmt_datetime(dt),
        None => none.to_string(),
    }
}
