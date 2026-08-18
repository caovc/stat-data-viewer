use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use duckdb::Connection;
use readstat::FileFormat;

use crate::sqlutil::quote_ident;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct TableReg {
    pub table_name: String,
    pub source_path: PathBuf,
    pub mtime_ms: i64,
    pub format: FileFormat,
    pub encoding: Option<String>,
    pub catalog_path: Option<PathBuf>,
    pub import_complete: bool,
}

#[allow(dead_code)]
pub struct ImportJob {
    pub cancel: Arc<AtomicBool>,
    pub table_name: String,
}

pub struct Session {
    pub db_path: PathBuf,
    write: Mutex<Connection>,
    pub tables: Mutex<HashMap<String, TableReg>>,
}

impl Session {
    pub fn new() -> anyhow_result::Result<Self, String> {
        let dir = std::env::temp_dir().join("stat-data-viewer");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let db_path = dir.join(format!("session-{}.duckdb", uuid::Uuid::new_v4()));
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS meta_datasets (
              table_name VARCHAR PRIMARY KEY,
              source_path VARCHAR,
              mtime_ms BIGINT,
              file_format VARCHAR,
              encoding VARCHAR,
              file_label VARCHAR,
              file_encoding VARCHAR,
              format_version INTEGER,
              row_count BIGINT,
              var_count INTEGER,
              catalog_path VARCHAR,
              import_complete BOOLEAN
            );
            CREATE TABLE IF NOT EXISTS meta_variables (
              table_name VARCHAR,
              var_index INTEGER,
              name VARCHAR,
              label VARCHAR,
              storage_type VARCHAR,
              display_format VARCHAR,
              measure VARCHAR,
              display_width INTEGER,
              decimals INTEGER,
              missing_rules VARCHAR,
              label_set VARCHAR
            );
            CREATE TABLE IF NOT EXISTS meta_value_labels (
              table_name VARCHAR,
              label_set VARCHAR,
              num_value DOUBLE,
              str_value VARCHAR,
              tag VARCHAR,
              label VARCHAR
            );
            "#,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self {
            db_path,
            write: Mutex::new(conn),
            tables: Mutex::new(HashMap::new()),
        })
    }

    pub fn write_conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.write.lock().expect("session write lock")
    }

    /// Clone a connection on the *same* DuckDB instance.
    ///
    /// Opening the session file a second time (`Connection::open`) starts a
    /// separate engine. That second engine either sees an empty catalog or
    /// aborts in the WAL writer — the UI then shows an empty grid with no error.
    pub fn clone_conn(&self) -> Result<Connection, String> {
        self.write
            .lock()
            .map_err(|e| format!("session write lock poisoned: {e}"))?
            .try_clone()
            .map_err(|e| e.to_string())
    }

    pub fn query_conn(&self) -> Result<Connection, String> {
        self.clone_conn()
    }

    pub fn unique_table_name(&self, base: &str) -> String {
        let tables = self.tables.lock().unwrap();
        if !tables.contains_key(base) {
            return base.to_string();
        }
        for i in 2..10_000 {
            let name = format!("{base}_{i}");
            if !tables.contains_key(&name) {
                return name;
            }
        }
        format!("{base}_{}", uuid::Uuid::new_v4().simple())
    }

    pub fn find_reuse(&self, path: &str, mtime_ms: i64) -> Option<TableReg> {
        let tables = self.tables.lock().unwrap();
        tables
            .values()
            .find(|t| t.source_path.to_string_lossy() == path && t.mtime_ms == mtime_ms)
            .cloned()
    }

    pub fn drop_table(&self, table: &str) -> Result<(), String> {
        let conn = self.write_conn();
        let q = quote_ident(table);
        conn.execute_batch(&format!(
            "DROP TABLE IF EXISTS {q};
             DELETE FROM meta_variables WHERE table_name = {};
             DELETE FROM meta_value_labels WHERE table_name = {};
             DELETE FROM meta_datasets WHERE table_name = {};",
            crate::sqlutil::quote_string(table),
            crate::sqlutil::quote_string(table),
            crate::sqlutil::quote_string(table),
        ))
        .map_err(|e| e.to_string())?;
        drop(conn);
        self.tables.lock().unwrap().remove(table);
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Ok(conn) = self.write.lock() {
            let _ = conn.execute("CHECKPOINT", []);
        }
        let _ = std::fs::remove_file(&self.db_path);
        let wal = PathBuf::from(format!("{}.wal", self.db_path.display()));
        let _ = std::fs::remove_file(wal);
    }
}

pub struct AppState {
    pub session: Session,
    pub jobs: Mutex<HashMap<String, ImportJob>>,
}

impl AppState {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            session: Session::new()?,
            jobs: Mutex::new(HashMap::new()),
        })
    }
}

pub fn file_mtime_ms(path: &std::path::Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// local alias so we don't add anyhow just for Result naming
mod anyhow_result {
    pub type Result<T, E> = std::result::Result<T, E>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_conn_sees_rows_written_on_session() {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (id INTEGER);
                 INSERT INTO ads VALUES (1), (2), (3);
                 INSERT INTO meta_datasets VALUES ('ads', '/tmp/ads.sav', 0, 'sav', NULL, NULL, NULL, NULL, 3, 1, NULL, true);
                 INSERT INTO meta_variables VALUES ('ads', 0, 'id', NULL, 'int32', NULL, NULL, NULL, NULL, '[]', NULL);",
            )
            .expect("seed imported table");
        }
        let q = session.query_conn().expect("query conn");
        let n: i64 = q
            .query_row("SELECT COUNT(*) FROM ads", [], |r| r.get(0))
            .expect("count imported rows");
        assert_eq!(n, 3, "query connection must see imported rows");
    }
}
