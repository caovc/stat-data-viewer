use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use duckdb::Connection;
use readstat::{
    parse_file, sanitize_table_name, BatchSink, DatasetMeta, FileFormat, ParseHooks, ParseOptions,
    StorageType,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::sqlutil::{quote_ident, quote_string};
use crate::state::{file_mtime_ms, AppState, ImportJob, TableReg};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportEvent {
    pub job_id: String,
    pub table_name: String,
    pub progress: f64,
    pub rows_imported: u64,
    pub preview_ready: bool,
    pub complete: bool,
    pub error: Option<String>,
}

struct DuckSink<'a> {
    conn: &'a Connection,
    table: String,
    app: AppHandle,
    job_id: String,
    rows: u64,
    preview_sent: bool,
    last_progress: f64,
}

impl DuckSink<'_> {
    fn emit(&self, preview: bool, complete: bool, error: Option<String>) {
        let _ = self.app.emit(
            "import-progress",
            ImportEvent {
                job_id: self.job_id.clone(),
                table_name: self.table.clone(),
                progress: self.last_progress,
                rows_imported: self.rows,
                preview_ready: preview,
                complete,
                error,
            },
        );
    }
}

impl BatchSink for DuckSink<'_> {
    fn on_metadata(&mut self, meta: &DatasetMeta) -> readstat::Result<()> {
        persist_metadata(self.conn, &self.table, meta).map_err(readstat::Error::msg)?;
        create_data_table(self.conn, &self.table, meta).map_err(readstat::Error::msg)?;
        Ok(())
    }

    fn on_batch(&mut self, batch: RecordBatch) -> readstat::Result<()> {
        let n = batch.num_rows() as u64;
        {
            let mut appender = self
                .conn
                .appender(&self.table)
                .map_err(|e| readstat::Error::msg(e.to_string()))?;
            appender
                .append_record_batch(batch)
                .map_err(|e| readstat::Error::msg(e.to_string()))?;
        }
        self.rows += n;
        let first = !self.preview_sent;
        if first {
            self.preview_sent = true;
        }
        self.emit(first || self.preview_sent, false, None);
        Ok(())
    }
}

fn duck_type(storage: StorageType) -> &'static str {
    match storage {
        StorageType::String => "VARCHAR",
        StorageType::Int32 => "INTEGER",
        StorageType::Float64 => "DOUBLE",
    }
}

fn create_data_table(conn: &Connection, table: &str, meta: &DatasetMeta) -> Result<(), String> {
    let cols = meta
        .variables
        .iter()
        .map(|v| format!("{} {}", quote_ident(&v.name), duck_type(v.storage_type)))
        .collect::<Vec<_>>()
        .join(", ");
    conn.execute_batch(&format!(
        "CREATE TABLE IF NOT EXISTS {} ({cols})",
        quote_ident(table)
    ))
    .map_err(|e| e.to_string())
}

fn persist_metadata(conn: &Connection, table: &str, meta: &DatasetMeta) -> Result<(), String> {
    let t = quote_string(table);
    conn.execute(
        &format!("DELETE FROM meta_variables WHERE table_name = {t}"),
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        &format!("DELETE FROM meta_value_labels WHERE table_name = {t}"),
        [],
    )
    .map_err(|e| e.to_string())?;

    for v in &meta.variables {
        conn.execute(
            "INSERT INTO meta_variables VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            duckdb::params![
                table,
                v.index,
                v.name,
                v.label,
                v.storage_type.as_str(),
                v.display_format,
                v.measure,
                v.display_width,
                v.decimals,
                serde_json::to_string(&v.missing_rules).unwrap_or_else(|_| "[]".into()),
                v.label_set,
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    for l in &meta.value_labels {
        conn.execute(
            "INSERT INTO meta_value_labels VALUES (?, ?, ?, ?, ?, ?)",
            duckdb::params![
                table,
                l.label_set,
                l.num_value,
                l.str_value,
                l.tag,
                l.label,
            ],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn upsert_dataset_row(
    conn: &Connection,
    table: &str,
    path: &str,
    mtime_ms: i64,
    format: FileFormat,
    encoding: Option<&str>,
    meta: Option<&DatasetMeta>,
    rows: i64,
    complete: bool,
) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO meta_datasets VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        duckdb::params![
            table,
            path,
            mtime_ms,
            format.as_str(),
            encoding,
            meta.and_then(|m| m.file.file_label.clone()),
            meta.and_then(|m| m.file.file_encoding.clone()),
            meta.and_then(|m| m.file.format_version),
            rows,
            meta.map(|m| m.variables.len() as i32),
            meta.and_then(|m| m.file.catalog_path.as_ref().map(|p| p.display().to_string())),
            complete,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn import_dataset(
    app: AppHandle,
    state: &AppState,
    job_id: String,
    path: String,
    encoding: Option<String>,
    format: Option<FileFormat>,
    catalog_path: Option<String>,
    cancel: Arc<AtomicBool>,
    table: Option<String>,
) -> Result<String, String> {
    let src = std::path::PathBuf::from(&path);
    if !src.exists() {
        return Err(format!("file not found: {path}"));
    }
    let resolved = format
        .or_else(|| FileFormat::from_path(&src))
        .ok_or_else(|| format!("unrecognized format: {path}"))?;
    if !resolved.is_dataset() {
        return Err(readstat::Error::CatalogIsNotDataset.to_string());
    }

    let mtime_ms = file_mtime_ms(&src);
    let table = match table {
        Some(name) => name,
        None => {
            let (reg, _) = state.session.reuse_or_reserve(
                &path,
                mtime_ms,
                &sanitize_table_name(&src),
                |table_name| TableReg {
                    table_name,
                    source_path: src.clone(),
                    mtime_ms,
                    format: resolved,
                    encoding: encoding.clone(),
                    catalog_path: catalog_path.as_ref().map(std::path::PathBuf::from),
                    import_complete: false,
                },
            );
            reg.table_name
        }
    };

    state.tables_insert(TableReg {
        table_name: table.clone(),
        source_path: src.clone(),
        mtime_ms,
        format: resolved,
        encoding: encoding.clone(),
        catalog_path: catalog_path.as_ref().map(std::path::PathBuf::from),
        import_complete: false,
    });

    let conn = state.session.clone_conn()?;
    let mut sink = DuckSink {
        conn: &conn,
        table: table.clone(),
        app: app.clone(),
        job_id: job_id.clone(),
        rows: 0,
        preview_sent: false,
        last_progress: 0.0,
    };

    let progress_job = job_id.clone();
    let progress_table = table.clone();
    let progress_app = app.clone();
    let progress_cb = |p: f64| {
        let _ = progress_app.emit(
            "import-progress",
            ImportEvent {
                job_id: progress_job.clone(),
                table_name: progress_table.clone(),
                progress: p,
                rows_imported: 0,
                preview_ready: false,
                complete: false,
                error: None,
            },
        );
    };
    let hooks = ParseHooks {
        cancel: Some(cancel.clone()),
        progress: Some(&progress_cb),
    };

    let opts = ParseOptions {
        encoding: encoding.clone(),
        format: Some(resolved),
        catalog_path: catalog_path.as_ref().map(std::path::PathBuf::from),
        batch_size: 2_000,
    };

    let parsed = parse_file(&src, opts, hooks, &mut sink);
    let still_open = state
        .session
        .tables
        .lock()
        .map(|tables| tables.contains_key(&table))
        .unwrap_or(false);
    if !still_open {
        drop(sink);
        drop(conn);
        let _ = state.session.drop_table(&table);
        return Ok(table);
    }
    let complete = !cancel.load(Ordering::Relaxed) && parsed.is_ok();
    let rows = sink.rows as i64;
    let preview_sent = sink.preview_sent;
    upsert_dataset_row(
        &conn,
        &table,
        &path,
        mtime_ms,
        resolved,
        encoding.as_deref(),
        parsed.as_ref().ok(),
        rows,
        complete,
    )?;

    if let Ok(mut tables) = state.session.tables.lock() {
        if let Some(reg) = tables.get_mut(&table) {
            reg.import_complete = complete;
        }
    }

    let result = match parsed {
        Ok(_) => {
            sink.emit(true, true, None);
            Ok(table)
        }
        Err(e) if matches!(e, readstat::Error::Cancelled) => {
            sink.emit(preview_sent, false, Some("cancelled".into()));
            Ok(table)
        }
        Err(e) => {
            let msg = e.to_string();
            sink.emit(preview_sent, false, Some(msg.clone()));
            Err(msg)
        }
    };
    drop(sink);
    drop(conn);
    result
}

impl AppState {
    pub fn tables_insert(&self, reg: TableReg) {
        self.session
            .tables
            .lock()
            .unwrap()
            .insert(reg.table_name.clone(), reg);
    }

    pub fn register_job(&self, job_id: String, table: String) -> Arc<AtomicBool> {
        let cancel = Arc::new(AtomicBool::new(false));
        self.jobs.lock().unwrap().insert(
            job_id,
            ImportJob {
                cancel: cancel.clone(),
                table_name: table,
            },
        );
        cancel
    }
}
