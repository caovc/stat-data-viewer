use std::sync::atomic::Ordering;

use readstat::FileFormat;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::export::{export_sql, table_select_sql, ExportFormat};
use crate::ingest::import_dataset;
use crate::query::{
    column_distinct, query_page, run_sql_page, DistinctResult, FilterNode, PageResult, SortSpec,
};
use crate::state::{file_mtime_ms, AppState};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenArgs {
    pub path: String,
    pub encoding: Option<String>,
    pub format: Option<String>,
    pub catalog_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenResult {
    pub job_id: String,
    pub table_name: String,
    pub reused: bool,
    pub import_complete: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryArgs {
    pub table: String,
    pub offset: u64,
    pub page_size: Option<u64>,
    pub sorts: Option<Vec<SortSpec>>,
    pub filters: Option<FilterNode>,
    pub hidden: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlArgs {
    pub sql: String,
    pub offset: u64,
    pub page_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportArgs {
    pub path: String,
    pub format: String,
    pub table: Option<String>,
    pub sql: Option<String>,
}

fn page_size(v: Option<u64>) -> u64 {
    v.unwrap_or(300).clamp(200, 500)
}

fn parse_format(name: Option<&str>) -> Result<Option<FileFormat>, String> {
    match name {
        None => Ok(None),
        Some(s) => FileFormat::from_name(s)
            .map(Some)
            .ok_or_else(|| format!("unknown format: {s}")),
    }
}

#[tauri::command]
pub fn open_dataset(app: AppHandle, state: State<AppState>, args: OpenArgs) -> Result<OpenResult, String> {
    let mtime = file_mtime_ms(std::path::Path::new(&args.path));
    if let Some(existing) = state.session.find_reuse(&args.path, mtime) {
        return Ok(OpenResult {
            job_id: String::new(),
            table_name: existing.table_name,
            reused: true,
            import_complete: existing.import_complete,
        });
    }

    let format = parse_format(args.format.as_deref())?;
    let src = std::path::PathBuf::from(&args.path);
    let table = state
        .session
        .unique_table_name(&readstat::sanitize_table_name(&src));
    let job_id = uuid::Uuid::new_v4().to_string();
    let job_out = job_id.clone();
    let table_out = table.clone();
    let cancel = state.register_job(job_id.clone(), table.clone());
    let app2 = app.clone();
    std::thread::spawn(move || {
        let state = app2.state::<AppState>();
        if let Err(error) = import_dataset(
            app2.clone(),
            &state,
            job_id.clone(),
            args.path,
            args.encoding,
            format,
            args.catalog_path,
            cancel,
            Some(table.clone()),
        ) {
            let _ = app2.emit(
                "import-progress",
                crate::ingest::ImportEvent {
                    job_id,
                    table_name: table,
                    progress: 0.0,
                    rows_imported: 0,
                    preview_ready: false,
                    complete: false,
                    error: Some(error),
                },
            );
        }
    });
    Ok(OpenResult {
        job_id: job_out,
        table_name: table_out,
        reused: false,
        import_complete: false,
    })
}

#[tauri::command]
pub fn reimport(app: AppHandle, state: State<AppState>, table: String, args: OpenArgs) -> Result<OpenResult, String> {
    let source = {
        let tables = state.session.tables.lock().unwrap();
        tables.get(&table).map(|t| t.source_path.clone())
    };
    let path = args.path.clone();
    let path = if path.is_empty() {
        source
            .ok_or_else(|| format!("unknown table {table}"))?
            .display()
            .to_string()
    } else {
        path
    };
    state.session.drop_table(&table)?;
    let mut args = args;
    args.path = path;
    open_dataset(app, state, args)
}

#[tauri::command]
pub fn cancel_import(state: State<AppState>, job_id: String) -> Result<(), String> {
    if let Some(job) = state.jobs.lock().unwrap().get(&job_id) {
        job.cancel.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command(rename = "query_page")]
pub fn query_page_cmd(state: State<AppState>, args: QueryArgs) -> Result<PageResult, String> {
    let conn = state.session.query_conn()?;
    query_page(
        &conn,
        &args.table,
        args.offset,
        page_size(args.page_size),
        args.sorts.as_deref().unwrap_or(&[]),
        args.filters.as_ref().unwrap_or(&FilterNode::empty_group()),
        args.hidden.as_deref().unwrap_or(&[]),
    )
}

#[tauri::command]
pub fn run_sql(state: State<AppState>, args: SqlArgs) -> Result<PageResult, String> {
    let conn = state.session.query_conn()?;
    run_sql_page(&conn, &args.sql, args.offset, page_size(args.page_size))
}

#[tauri::command]
pub fn export(state: State<AppState>, args: ExportArgs) -> Result<(), String> {
    let conn = state.session.query_conn()?;
    let format = ExportFormat::from_name(&args.format)?;
    let sql = if let Some(sql) = args.sql.filter(|s| !s.trim().is_empty()) {
        sql
    } else if let Some(table) = args.table {
        table_select_sql(&table)
    } else {
        return Err("export requires table or sql".into());
    };
    export_sql(&conn, &sql, &args.path, format)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataOut {
    pub table_name: String,
    pub source_path: Option<String>,
    pub file_format: Option<String>,
    pub encoding: Option<String>,
    pub file_label: Option<String>,
    pub format_version: Option<i32>,
    pub row_count: Option<i64>,
    pub var_count: Option<i32>,
    pub catalog_path: Option<String>,
    pub import_complete: bool,
    pub variables: Vec<Value>,
    pub value_labels: Vec<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistinctArgs {
    pub table: String,
    pub column: String,
    pub limit: Option<u64>,
}

#[tauri::command(rename = "column_distinct")]
pub fn column_distinct_cmd(state: State<AppState>, args: DistinctArgs) -> Result<DistinctResult, String> {
    let conn = state.session.query_conn()?;
    column_distinct(&conn, &args.table, &args.column, args.limit.unwrap_or(500))
}

#[tauri::command]
pub fn get_metadata(state: State<AppState>, table: String) -> Result<MetadataOut, String> {
    let conn = state.session.query_conn()?;
    let header = conn.query_row(
        "SELECT source_path, file_format, encoding, file_label, format_version, row_count, var_count, catalog_path, import_complete
         FROM meta_datasets WHERE table_name = ?",
        [&table],
        |r| {
            Ok((
                r.get::<_, Option<String>>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<i32>>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, Option<i32>>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, bool>(8).unwrap_or(true),
            ))
        },
    );
    let (source_path, file_format, encoding, file_label, format_version, row_count, var_count, catalog_path, import_complete) =
        header.unwrap_or((None, None, None, None, None, None, None, None, false));

    let mut vars = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT var_index, name, label, storage_type, display_format, measure, display_width, decimals, missing_rules, label_set
             FROM meta_variables WHERE table_name = ? ORDER BY var_index",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([&table], |r| {
        Ok(serde_json::json!({
            "index": r.get::<_, i32>(0)?,
            "name": r.get::<_, String>(1)?,
            "label": r.get::<_, Option<String>>(2)?,
            "storageType": r.get::<_, String>(3)?,
            "displayFormat": r.get::<_, Option<String>>(4)?,
            "measure": r.get::<_, Option<String>>(5)?,
            "displayWidth": r.get::<_, Option<i32>>(6)?,
            "decimals": r.get::<_, Option<i32>>(7)?,
            "missingRules": r.get::<_, Option<String>>(8)?,
            "labelSet": r.get::<_, Option<String>>(9)?,
        }))
    }).map_err(|e| e.to_string())?;
    for row in rows {
        vars.push(row.map_err(|e| e.to_string())?);
    }

    let mut labels = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT label_set, num_value, str_value, tag, label FROM meta_value_labels WHERE table_name = ?",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([&table], |r| {
            Ok(serde_json::json!({
                "labelSet": r.get::<_, String>(0)?,
                "numValue": r.get::<_, Option<f64>>(1)?,
                "strValue": r.get::<_, Option<String>>(2)?,
                "tag": r.get::<_, Option<String>>(3)?,
                "label": r.get::<_, String>(4)?,
            }))
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        labels.push(row.map_err(|e| e.to_string())?);
    }

    Ok(MetadataOut {
        table_name: table,
        source_path,
        file_format,
        encoding,
        file_label,
        format_version,
        row_count,
        var_count,
        catalog_path,
        import_complete,
        variables: vars,
        value_labels: labels,
    })
}

#[tauri::command]
pub fn list_datasets(state: State<AppState>) -> Result<Vec<String>, String> {
    let tables = state.session.tables.lock().unwrap();
    Ok(tables.keys().cloned().collect())
}
