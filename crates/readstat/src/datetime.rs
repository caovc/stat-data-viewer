use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

use crate::types::Origin;

/// SAS/Stata date epoch: 1960-01-01.
pub const SAS_EPOCH: NaiveDate = match NaiveDate::from_ymd_opt(1960, 1, 1) {
    Some(d) => d,
    None => panic!("valid date"),
};

/// SPSS datetime epoch: 1582-10-14 00:00:00 (Gregorian calendar start).
pub const SPSS_EPOCH: NaiveDateTime = match NaiveDate::from_ymd_opt(1582, 10, 14) {
    Some(d) => match d.and_hms_opt(0, 0, 0) {
        Some(dt) => dt,
        None => panic!("valid time"),
    },
    None => panic!("valid date"),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateKind {
    Date,
    DateTime,
    Time,
}

/// Classify a vendor display format so the UI can format raw stored numbers.
pub fn classify_format(origin: Origin, format: &str) -> Option<DateKind> {
    let raw = format.trim();
    if raw.is_empty() {
        return None;
    }
    match origin {
        Origin::Sas => classify_sas(raw),
        Origin::Spss => classify_spss(raw),
        Origin::Stata => classify_stata(raw),
    }
}

fn strip_width(fmt: &str) -> String {
    let mut out = String::new();
    for ch in fmt.chars() {
        if ch.is_ascii_alphabetic() || ch == '%' {
            out.push(ch.to_ascii_uppercase());
        } else if ch == '.' {
            break;
        }
    }
    out
}

fn classify_sas(fmt: &str) -> Option<DateKind> {
    let key = strip_width(fmt);
    const DATES: &[&str] = &[
        "DATE", "DDMMYY", "MMDDYY", "YYMMDD", "YYMM", "MMYY", "MONYY", "YEAR", "YYQ", "WEEKDATE",
        "WORDDATE", "WEEKDATX", "WORDDATX", "JULIAN", "NENGO",
    ];
    const DATETIMES: &[&str] = &["DATETIME", "B8601DT", "E8601DT", "DTDATE", "DTMONYY"];
    const TIMES: &[&str] = &["TIME", "TOD", "HHMM", "HOUR", "MMSS", "TIMEAMPM"];
    if DATETIMES.iter().any(|p| key.starts_with(p)) {
        return Some(DateKind::DateTime);
    }
    if DATES.iter().any(|p| key.starts_with(p)) {
        return Some(DateKind::Date);
    }
    if TIMES.iter().any(|p| key.starts_with(p)) {
        return Some(DateKind::Time);
    }
    None
}

fn classify_spss(fmt: &str) -> Option<DateKind> {
    let key = strip_width(fmt);
    const DATES: &[&str] = &[
        "DATE", "ADATE", "EDATE", "SDATE", "JDATE", "QYR", "MOYR", "WKYR", "WEEKDAY", "MONTH",
        "DATE11", "DATE9",
    ];
    const DATETIMES: &[&str] = &["DATETIME", "YMDHMS"];
    const TIMES: &[&str] = &["TIME", "DTIME"];
    if DATETIMES.iter().any(|p| key.starts_with(p)) {
        return Some(DateKind::DateTime);
    }
    if TIMES.iter().any(|p| key.starts_with(p)) {
        return Some(DateKind::Time);
    }
    if DATES.iter().any(|p| key.starts_with(p)) {
        return Some(DateKind::Date);
    }
    None
}

fn classify_stata(fmt: &str) -> Option<DateKind> {
    let key = fmt.trim().to_ascii_lowercase();
    if key.starts_with("%tc") || key.starts_with("%tC") {
        return Some(DateKind::DateTime);
    }
    if key.starts_with("%td")
        || key.starts_with("%tw")
        || key.starts_with("%tm")
        || key.starts_with("%tq")
        || key.starts_with("%th")
        || key.starts_with("%ty")
    {
        return Some(DateKind::Date);
    }
    if key.starts_with("%tc") {
        return Some(DateKind::DateTime);
    }
    None
}

pub fn decode_raw_datetime(origin: Origin, format: &str, value: f64) -> Option<(DateKind, NaiveDateTime)> {
    if !value.is_finite() {
        return None;
    }
    let kind = classify_format(origin, format)?;
    let dt = match (origin, kind) {
        (Origin::Sas | Origin::Stata, DateKind::Date) => {
            let days = value.round() as i64;
            SAS_EPOCH
                .checked_add_signed(Duration::days(days))?
                .and_hms_opt(0, 0, 0)?
        }
        (Origin::Sas, DateKind::DateTime) => {
            let millis = (value * 1000.0).round() as i64;
            SAS_EPOCH
                .and_hms_opt(0, 0, 0)?
                .checked_add_signed(Duration::milliseconds(millis))?
        }
        (Origin::Stata, DateKind::DateTime) => {
            let millis = value.round() as i64;
            SAS_EPOCH
                .and_hms_opt(0, 0, 0)?
                .checked_add_signed(Duration::milliseconds(millis))?
        }
        (Origin::Spss, DateKind::Date | DateKind::DateTime) => {
            let millis = (value * 1000.0).round() as i64;
            SPSS_EPOCH.checked_add_signed(Duration::milliseconds(millis))?
        }
        (_, DateKind::Time) => {
            let secs = value.round() as i64;
            let day = 24 * 3600;
            let secs = ((secs % day) + day) % day;
            let time = NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, 0)?;
            NaiveDate::from_ymd_opt(1970, 1, 1)?.and_time(time)
        }
    };
    Some((kind, dt))
}

pub fn format_raw_number(origin: Origin, format: &str, value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    let kind = classify_format(origin, format)?;
    match (origin, kind) {
        (Origin::Sas | Origin::Stata, DateKind::Date) => format_sas_date(value, format),
        (Origin::Sas, DateKind::DateTime) => format_sas_datetime(value),
        (Origin::Stata, DateKind::DateTime) => format_stata_datetime(value),
        (Origin::Sas | Origin::Stata, DateKind::Time) => format_day_time(value),
        (Origin::Spss, DateKind::Date | DateKind::DateTime) => format_spss_datetime(value, kind),
        (Origin::Spss, DateKind::Time) => format_day_time(value),
    }
}

fn format_sas_date(days: f64, format: &str) -> Option<String> {
    let days = days.round() as i64;
    let date = SAS_EPOCH.checked_add_signed(Duration::days(days))?;
    let key = strip_width(format);
    Some(if key.starts_with("YYMMDD") {
        date.format("%Y-%m-%d").to_string()
    } else if key.starts_with("DDMMYY") {
        date.format("%d/%m/%Y").to_string()
    } else if key.starts_with("MMDDYY") {
        date.format("%m/%d/%Y").to_string()
    } else if key.starts_with("MONYY") {
        date.format("%b%Y").to_string()
    } else if key.starts_with("YEAR") {
        date.format("%Y").to_string()
    } else {
        date.format("%Y-%m-%d").to_string()
    })
}

fn format_sas_datetime(seconds: f64) -> Option<String> {
    let millis = (seconds * 1000.0).round() as i64;
    let dt = SAS_EPOCH
        .and_hms_opt(0, 0, 0)?
        .checked_add_signed(Duration::milliseconds(millis))?;
    Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn format_stata_datetime(millis: f64) -> Option<String> {
    let millis = millis.round() as i64;
    let dt = SAS_EPOCH
        .and_hms_opt(0, 0, 0)?
        .checked_add_signed(Duration::milliseconds(millis))?;
    Some(dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
}

fn format_spss_datetime(seconds: f64, kind: DateKind) -> Option<String> {
    let millis = (seconds * 1000.0).round() as i64;
    let dt = SPSS_EPOCH.checked_add_signed(Duration::milliseconds(millis))?;
    Some(match kind {
        DateKind::Date => dt.date().format("%Y-%m-%d").to_string(),
        DateKind::DateTime => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        DateKind::Time => dt.time().format("%H:%M:%S").to_string(),
    })
}

fn format_day_time(seconds: f64) -> Option<String> {
    let secs = seconds.round() as i64;
    let day = 24 * 3600;
    let secs = ((secs % day) + day) % day;
    let time = NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, 0)?;
    Some(time.format("%H:%M:%S").to_string())
}

pub fn parse_filter_date_to_raw(origin: Origin, format: &str, text: &str) -> Option<f64> {
    let kind = classify_format(origin, format)?;
    let trimmed = text.trim();
    match (origin, kind) {
        (Origin::Sas | Origin::Stata, DateKind::Date) => {
            let date = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").ok()?;
            Some((date - SAS_EPOCH).num_days() as f64)
        }
        (Origin::Sas, DateKind::DateTime) => {
            let dt = parse_naive_dt(trimmed)?;
            Some((dt - SAS_EPOCH.and_hms_opt(0, 0, 0)?).num_seconds() as f64)
        }
        (Origin::Stata, DateKind::DateTime) => {
            let dt = parse_naive_dt(trimmed)?;
            Some((dt - SAS_EPOCH.and_hms_opt(0, 0, 0)?).num_milliseconds() as f64)
        }
        (Origin::Spss, DateKind::Date | DateKind::DateTime) => {
            let dt = if kind == DateKind::Date {
                NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
                    .ok()?
                    .and_hms_opt(0, 0, 0)?
            } else {
                parse_naive_dt(trimmed)?
            };
            Some((dt - SPSS_EPOCH).num_seconds() as f64)
        }
        (_, DateKind::Time) => {
            let t = NaiveTime::parse_from_str(trimmed, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(trimmed, "%H:%M"))
                .ok()?;
            Some(t.signed_duration_since(NaiveTime::from_hms_opt(0, 0, 0)?).num_seconds() as f64)
        }
    }
}

fn parse_naive_dt(text: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S")
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S").ok())
        .or_else(|| {
            NaiveDate::parse_from_str(text, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sas_date_epoch() {
        let formatted = format_raw_number(Origin::Sas, "DATE9.", 0.0).unwrap();
        assert_eq!(formatted, "1960-01-01");
        let formatted = format_raw_number(Origin::Sas, "DATE9.", 1.0).unwrap();
        assert_eq!(formatted, "1960-01-02");
    }

    #[test]
    fn spss_date_epoch() {
        let formatted = format_raw_number(Origin::Spss, "DATE11", 0.0).unwrap();
        assert_eq!(formatted, "1582-10-14");
    }

    #[test]
    fn stata_tc_millis() {
        let formatted = format_raw_number(Origin::Stata, "%tc", 0.0).unwrap();
        assert!(formatted.starts_with("1960-01-01"));
    }

    #[test]
    fn filter_roundtrip_sas_date() {
        let raw = parse_filter_date_to_raw(Origin::Sas, "DATE9.", "2020-01-15").unwrap();
        assert_eq!(
            format_raw_number(Origin::Sas, "DATE9.", raw).unwrap(),
            "2020-01-15"
        );
    }

    #[test]
    fn decode_sas_date_2020() {
        let (kind, dt) = decode_raw_datetime(Origin::Sas, "DATE9.", 21915.0).unwrap();
        assert_eq!(kind, DateKind::Date);
        assert_eq!(dt.date(), NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
    }
}
