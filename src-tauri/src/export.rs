use std::fs::File;
use std::io::{BufWriter, Write};

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use duckdb::types::{TimeUnit, ValueRef};
use duckdb::Connection;
use readstat::{decode_raw_datetime, format_raw_number, DateKind, Origin};
use rust_xlsxwriter::{ExcelDateTime, Format, Workbook};

use crate::query::{describe_sql_columns, ColumnOut};
use crate::sqlutil::{quote_ident, quote_string, strip_trailing_semicolon};

#[derive(Clone, Copy)]
pub enum ExportFormat {
    Csv,
    Parquet,
    Excel,
}

impl ExportFormat {
    pub fn from_name(name: &str) -> Result<Self, String> {
        match name.to_ascii_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "parquet" | "pq" => Ok(Self::Parquet),
            "xlsx" | "excel" => Ok(Self::Excel),
            other => Err(format!("unsupported export format: {other}")),
        }
    }
}

pub fn export_sql(
    conn: &Connection,
    sql: &str,
    dest: &str,
    format: ExportFormat,
) -> Result<(), String> {
    let inner = strip_trailing_semicolon(sql);
    match format {
        ExportFormat::Csv => export_csv(conn, inner, dest),
        ExportFormat::Parquet => conn
            .execute(
                &format!("COPY ({inner}) TO {} (FORMAT PARQUET)", quote_string(dest)),
                [],
            )
            .map(|_| ())
            .map_err(|e| e.to_string()),
        ExportFormat::Excel => export_excel(conn, inner, dest),
    }
}

pub fn table_select_sql(table: &str) -> String {
    format!("SELECT * FROM {}", quote_ident(table))
}

fn export_csv(conn: &Connection, sql: &str, dest: &str) -> Result<(), String> {
    let columns = describe_sql_columns(conn, sql)?;
    if columns.iter().any(|c| c.is_datetime) {
        write_formatted_csv(conn, sql, dest, &columns)
    } else {
        conn.execute(
            &format!(
                "COPY ({sql}) TO {} (FORMAT CSV, HEADER true)",
                quote_string(dest)
            ),
            [],
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
    }
}

fn write_formatted_csv(
    conn: &Connection,
    sql: &str,
    dest: &str,
    columns: &[ColumnOut],
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(&format!("SELECT * FROM ({sql}) AS _sdv_q"))
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    let mut out = BufWriter::new(File::create(dest).map_err(|e| e.to_string())?);
    writeln!(
        out,
        "{}",
        columns
            .iter()
            .map(|c| csv_escape(&c.name))
            .collect::<Vec<_>>()
            .join(",")
    )
    .map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let line = columns
            .iter()
            .enumerate()
            .map(|(i, col)| csv_escape(&cell_csv(row, i, col)))
            .collect::<Vec<_>>()
            .join(",");
        writeln!(out, "{line}").map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn cell_csv(row: &duckdb::Row<'_>, i: usize, col: &ColumnOut) -> String {
    match read_export_cell(row, i, col) {
        ExportCell::Empty => String::new(),
        ExportCell::Number(n) => n.to_string(),
        ExportCell::Integer(n) => n.to_string(),
        ExportCell::Text(s) => s,
        ExportCell::Date { kind, dt } => format_export_date(kind, dt),
    }
}

fn export_excel(conn: &Connection, sql: &str, dest: &str) -> Result<(), String> {
    let columns = describe_sql_columns(conn, sql)?;
    let mut stmt = conn
        .prepare(&format!("SELECT * FROM ({sql}) AS _sdv_q"))
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    let mut workbook = Workbook::new();
    let header_fmt = Format::new().set_bold();
    let date_fmt = Format::new().set_num_format("yyyy-mm-dd");
    let datetime_fmt = Format::new().set_num_format("yyyy-mm-dd hh:mm:ss");
    let time_fmt = Format::new().set_num_format("hh:mm:ss");
    let sheet = workbook.add_worksheet();
    sheet.set_name("export").map_err(|e| e.to_string())?;
    for (c, col) in columns.iter().enumerate() {
        sheet
            .write_string_with_format(0, c as u16, &col.name, &header_fmt)
            .map_err(|e| e.to_string())?;
    }
    let mut r = 1u32;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        for (c, col) in columns.iter().enumerate() {
            write_excel_cell(sheet, r, c as u16, row, c, col, &date_fmt, &datetime_fmt, &time_fmt)?;
        }
        r += 1;
        if r > 1_048_575 {
            break;
        }
    }
    workbook.save(dest).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_excel_cell(
    sheet: &mut rust_xlsxwriter::Worksheet,
    r: u32,
    c: u16,
    row: &duckdb::Row<'_>,
    i: usize,
    col: &ColumnOut,
    date_fmt: &Format,
    datetime_fmt: &Format,
    time_fmt: &Format,
) -> Result<(), String> {
    match read_export_cell(row, i, col) {
        ExportCell::Empty => Ok(()),
        ExportCell::Number(v) => sheet.write_number(r, c, v).map(|_| ()).map_err(|e| e.to_string()),
        ExportCell::Integer(v) => sheet
            .write_number(r, c, v as f64)
            .map(|_| ())
            .map_err(|e| e.to_string()),
        ExportCell::Text(v) => sheet.write_string(r, c, &v).map(|_| ()).map_err(|e| e.to_string()),
        ExportCell::Date { kind, dt } => {
            if dt.year() < 1900 || dt.year() > 9999 {
                return sheet
                    .write_string(r, c, &format_export_date(kind, dt))
                    .map(|_| ())
                    .map_err(|e| e.to_string());
            }
            let (excel, fmt) = excel_date(kind, dt, date_fmt, datetime_fmt, time_fmt)?;
            sheet
                .write_datetime_with_format(r, c, &excel, fmt)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
    }
}

fn excel_date<'a>(
    kind: DateKind,
    dt: NaiveDateTime,
    date_fmt: &'a Format,
    datetime_fmt: &'a Format,
    time_fmt: &'a Format,
) -> Result<(ExcelDateTime, &'a Format), String> {
    match kind {
        DateKind::Date => {
            let d = dt.date();
            Ok((
                ExcelDateTime::from_ymd(d.year() as u16, d.month() as u8, d.day() as u8)
                    .map_err(|e| e.to_string())?,
                date_fmt,
            ))
        }
        DateKind::DateTime => {
            let d = dt.date();
            let t = dt.time();
            Ok((
                ExcelDateTime::from_ymd(d.year() as u16, d.month() as u8, d.day() as u8)
                    .map_err(|e| e.to_string())?
                    .and_hms(t.hour() as u16, t.minute() as u8, f64::from(t.second()))
                    .map_err(|e| e.to_string())?,
                datetime_fmt,
            ))
        }
        DateKind::Time => {
            let t = dt.time();
            Ok((
                ExcelDateTime::from_hms(t.hour() as u16, t.minute() as u8, f64::from(t.second()))
                    .map_err(|e| e.to_string())?,
                time_fmt,
            ))
        }
    }
}

enum ExportCell {
    Empty,
    Number(f64),
    Integer(i64),
    Text(String),
    Date { kind: DateKind, dt: NaiveDateTime },
}

fn column_origin(col: &ColumnOut) -> Origin {
    match col.origin.as_str() {
        "spss" => Origin::Spss,
        "stata" => Origin::Stata,
        _ => Origin::Sas,
    }
}

fn format_export_date(kind: DateKind, dt: NaiveDateTime) -> String {
    match kind {
        DateKind::Date => dt.format("%Y-%m-%d").to_string(),
        DateKind::DateTime => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        DateKind::Time => dt.format("%H:%M:%S").to_string(),
    }
}

fn read_export_cell(row: &duckdb::Row<'_>, i: usize, col: &ColumnOut) -> ExportCell {
    if col.is_datetime {
        if let Some(fmt) = col.display_format.as_deref() {
            if let Ok(Some(n)) = row.get::<_, Option<f64>>(i) {
                let origin = column_origin(col);
                if let Some((kind, dt)) = decode_raw_datetime(origin, fmt, n) {
                    return ExportCell::Date { kind, dt };
                }
                if let Some(s) = format_raw_number(origin, fmt, n) {
                    return ExportCell::Text(s);
                }
            }
        }
        if let Ok(value) = row.get_ref(i) {
            if let Some(cell) = native_date_cell(value) {
                return cell;
            }
        }
    }
    if let Ok(Some(v)) = row.get::<_, Option<f64>>(i) {
        return ExportCell::Number(v);
    }
    if let Ok(Some(v)) = row.get::<_, Option<i64>>(i) {
        return ExportCell::Integer(v);
    }
    if let Ok(Some(v)) = row.get::<_, Option<String>>(i) {
        return ExportCell::Text(v);
    }
    ExportCell::Empty
}

fn native_date_cell(value: ValueRef<'_>) -> Option<ExportCell> {
    match value {
        ValueRef::Null => Some(ExportCell::Empty),
        ValueRef::Date32(days) => {
            let date = NaiveDate::from_ymd_opt(1970, 1, 1)?
                .checked_add_signed(Duration::days(i64::from(days)))?;
            Some(ExportCell::Date {
                kind: DateKind::Date,
                dt: date.and_hms_opt(0, 0, 0)?,
            })
        }
        ValueRef::Timestamp(unit, raw) => timestamp_cell(unit, raw, DateKind::DateTime),
        ValueRef::Time64(unit, raw) => {
            let micros = unit.to_micros(raw);
            let secs = micros.div_euclid(1_000_000);
            let nsecs = (micros.rem_euclid(1_000_000) * 1000) as u32;
            let day = 24 * 3600;
            let secs = ((secs % day) + day) % day;
            let time = NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, nsecs)?;
            Some(ExportCell::Date {
                kind: DateKind::Time,
                dt: NaiveDate::from_ymd_opt(1970, 1, 1)?.and_time(time),
            })
        }
        ValueRef::Text(bytes) => {
            let text = std::str::from_utf8(bytes).ok()?.trim();
            parse_text_date(text)
        }
        _ => None,
    }
}

fn timestamp_cell(unit: TimeUnit, raw: i64, kind: DateKind) -> Option<ExportCell> {
    let micros = unit.to_micros(raw);
    let secs = micros.div_euclid(1_000_000);
    let nsecs = (micros.rem_euclid(1_000_000) * 1000) as u32;
    let dt = chrono::DateTime::from_timestamp(secs, nsecs)?.naive_utc();
    Some(ExportCell::Date { kind, dt })
}

fn parse_text_date(text: &str) -> Option<ExportCell> {
    if let Ok(date) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        return Some(ExportCell::Date {
            kind: DateKind::Date,
            dt: date.and_hms_opt(0, 0, 0)?,
        });
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S"))
    {
        return Some(ExportCell::Date {
            kind: DateKind::DateTime,
            dt,
        });
    }
    if let Ok(time) = NaiveTime::parse_from_str(text, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(text, "%H:%M"))
    {
        return Some(ExportCell::Date {
            kind: DateKind::Time,
            dt: NaiveDate::from_ymd_opt(1970, 1, 1)?.and_time(time),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Session;

    fn seed_ads() -> Session {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (id INTEGER, name VARCHAR);
                 INSERT INTO ads VALUES (1, 'a'), (2, 'b');",
            )
            .expect("seed");
        }
        session
    }

    fn seed_dated() -> Session {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (id INTEGER, visit DOUBLE, native_dt DATE);
                 INSERT INTO ads VALUES (1, 21915, DATE '2020-01-01');
                 INSERT INTO meta_datasets VALUES ('ads', '/tmp/ads.sas7bdat', 0, 'sas7bdat', NULL, NULL, NULL, NULL, 1, 3, NULL, true);
                 INSERT INTO meta_variables VALUES ('ads', 0, 'id', NULL, 'int32', NULL, NULL, NULL, NULL, '[]', NULL);
                 INSERT INTO meta_variables VALUES ('ads', 1, 'visit', 'Visit date', 'float64', 'DATE9.', NULL, NULL, NULL, '[]', NULL);",
            )
            .expect("seed");
        }
        session
    }

    #[test]
    fn export_csv_from_select() {
        let session = seed_ads();
        let conn = session.query_conn().expect("query conn");
        let dest = session.db_path.with_file_name("export-test.csv");
        export_sql(&conn, "SELECT * FROM ads", dest.to_str().unwrap(), ExportFormat::Csv)
            .expect("csv export");
        let text = std::fs::read_to_string(&dest).expect("csv");
        assert!(text.contains("id"), "{text}");
        assert!(text.contains("a"), "{text}");
        let _ = std::fs::remove_file(dest);
    }

    #[test]
    fn export_parquet_from_select() {
        let session = seed_ads();
        let conn = session.query_conn().expect("query conn");
        let dest = session.db_path.with_file_name("export-test.parquet");
        export_sql(
            &conn,
            "SELECT * FROM ads",
            dest.to_str().unwrap(),
            ExportFormat::Parquet,
        )
        .expect("parquet export");
        assert!(dest.exists());
        let _ = std::fs::remove_file(dest);
    }

    #[test]
    fn export_excel_from_select() {
        let session = seed_ads();
        let conn = session.query_conn().expect("query conn");
        let dest = session.db_path.with_file_name("export-test.xlsx");
        export_sql(
            &conn,
            "SELECT * FROM ads",
            dest.to_str().unwrap(),
            ExportFormat::Excel,
        )
        .expect("excel export");
        assert!(dest.exists());
        let _ = std::fs::remove_file(dest);
    }

    #[test]
    fn export_csv_writes_formatted_dates() {
        let session = seed_dated();
        let conn = session.query_conn().expect("query conn");
        let dest = session.db_path.with_file_name("export-dates.csv");
        export_sql(&conn, "SELECT * FROM ads", dest.to_str().unwrap(), ExportFormat::Csv)
            .expect("csv export");
        let text = std::fs::read_to_string(&dest).expect("csv");
        assert!(text.contains("2020-01-01"), "{text}");
        assert!(!text.contains("21915"), "{text}");
        let _ = std::fs::remove_file(dest);
    }

    #[test]
    fn export_excel_writes_excel_dates_not_raw_numbers() {
        let session = seed_dated();
        let conn = session.query_conn().expect("query conn");
        let dest = session.db_path.with_file_name("export-dates.xlsx");
        export_sql(
            &conn,
            "SELECT * FROM ads",
            dest.to_str().unwrap(),
            ExportFormat::Excel,
        )
        .expect("excel export");
        let sheet = unzip_entry(&dest, "xl/worksheets/sheet1.xml");
        let styles = unzip_entry(&dest, "xl/styles.xml");
        assert!(!sheet.contains("21915"), "{sheet}");
        assert!(
            styles.contains("yyyy-mm-dd"),
            "excel date number format missing: {styles}"
        );
        let _ = std::fs::remove_file(dest);
    }

    fn unzip_entry(path: &std::path::Path, name: &str) -> String {
        let out = std::process::Command::new("unzip")
            .args(["-p", path.to_str().unwrap(), name])
            .output()
            .expect("unzip");
        assert!(out.status.success(), "unzip {name} failed");
        String::from_utf8_lossy(&out.stdout).into_owned()
    }
}
