use duckdb::Connection;
use rust_xlsxwriter::{Format, Workbook};

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
        ExportFormat::Csv => {
            conn.execute(
                &format!(
                    "COPY ({inner}) TO {} (FORMAT CSV, HEADER true)",
                    quote_string(dest)
                ),
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        ExportFormat::Parquet => {
            conn.execute(
                &format!("COPY ({inner}) TO {} (FORMAT PARQUET)", quote_string(dest)),
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        ExportFormat::Excel => export_excel(conn, inner, dest)?,
    }
    Ok(())
}

pub fn table_select_sql(table: &str) -> String {
    format!("SELECT * FROM {}", quote_ident(table))
}

fn export_excel(conn: &Connection, sql: &str, dest: &str) -> Result<(), String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    let names = rows
        .as_ref()
        .map(|s| s.column_names())
        .ok_or_else(|| "query produced no statement schema".to_string())?;
    let mut workbook = Workbook::new();
    let header_fmt = Format::new().set_bold();
    let sheet = workbook.add_worksheet();
    sheet.set_name("export").map_err(|e| e.to_string())?;
    for (c, name) in names.iter().enumerate() {
        sheet
            .write_string_with_format(0, c as u16, name, &header_fmt)
            .map_err(|e| e.to_string())?;
    }
    let mut r = 1u32;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        for c in 0..names.len() {
            if let Ok(Some(v)) = row.get::<_, Option<f64>>(c) {
                sheet.write_number(r, c as u16, v).map_err(|e| e.to_string())?;
            } else if let Ok(Some(v)) = row.get::<_, Option<i64>>(c) {
                sheet
                    .write_number(r, c as u16, v as f64)
                    .map_err(|e| e.to_string())?;
            } else if let Ok(Some(v)) = row.get::<_, Option<String>>(c) {
                sheet.write_string(r, c as u16, &v).map_err(|e| e.to_string())?;
            }
        }
        r += 1;
        if r > 1_048_575 {
            break;
        }
    }
    workbook.save(dest).map_err(|e| e.to_string())?;
    Ok(())
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
}
