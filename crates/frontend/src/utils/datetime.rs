use chrono::{DateTime, Local, NaiveDateTime, Utc};

const DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

pub fn datetime_to_input(datetime: &DateTime<Utc>) -> String {
    datetime
        .with_timezone(&Local)
        .naive_local()
        .format(DATE_FORMAT)
        .to_string()
}

pub fn input_to_datetime(input: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(input, DATE_FORMAT)
        .ok()
        .and_then(|dt| dt.and_local_timezone(Local).single())
        .map(|dt| dt.to_utc())
}

pub fn fmt_datetime(datetime: &DateTime<Utc>) -> String {
    datetime.with_timezone(&Local).to_rfc2822()
}

pub fn fmt_datetime_opt(datetime: Option<&DateTime<Utc>>) -> String {
    match datetime {
        Some(dt) => fmt_datetime(dt),
        None => "None".to_string(),
    }
}
