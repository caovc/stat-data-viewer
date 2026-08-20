use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
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

    /// Atomically reuse an already-imported (or reserved) table, or occupy a unique name.
    ///
    /// Name allocation must happen before the import thread starts. Otherwise two
    /// same-stem files opened together both receive `adsl` and overwrite each other.
    pub fn reuse_or_reserve(
        &self,
        path: &str,
        mtime_ms: i64,
        base: &str,
        make: impl FnOnce(String) -> TableReg,
    ) -> (TableReg, bool) {
        let mut tables = self.tables.lock().unwrap();
        if let Some(existing) = tables
            .values()
            .find(|t| same_source_path(&t.source_path, path) && t.mtime_ms == mtime_ms)
            .cloned()
        {
            return (existing, true);
        }
        let name = next_unique_name(&tables, base);
        let reg = make(name.clone());
        tables.insert(name, reg.clone());
        (reg, false)
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

    pub fn cancel_jobs_for_table(&self, table: &str) {
        let jobs = self.jobs.lock().unwrap();
        for job in jobs.values() {
            if job.table_name == table {
                job.cancel.store(true, Ordering::Relaxed);
            }
        }
    }
}

fn next_unique_name(tables: &HashMap<String, TableReg>, base: &str) -> String {
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

fn same_source_path(stored: &Path, path: &str) -> bool {
    stored == Path::new(path)
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

    fn placeholder(path: &str, table_name: String) -> TableReg {
        TableReg {
            table_name,
            source_path: PathBuf::from(path),
            mtime_ms: 1,
            format: FileFormat::Sav,
            encoding: None,
            catalog_path: None,
            import_complete: false,
        }
    }

    #[test]
    fn same_stem_files_reserve_distinct_table_names() {
        let session = Session::new().expect("session");
        let (a, reused_a) = session.reuse_or_reserve("/study/a/adsl.sav", 1, "adsl", |name| {
            placeholder("/study/a/adsl.sav", name)
        });
        let (b, reused_b) = session.reuse_or_reserve("/study/b/adsl.sav", 1, "adsl", |name| {
            placeholder("/study/b/adsl.sav", name)
        });
        assert!(!reused_a);
        assert!(!reused_b);
        assert_eq!(a.table_name, "adsl");
        assert_eq!(b.table_name, "adsl_2");
        assert_ne!(a.table_name, b.table_name);
    }

    #[test]
    fn same_path_and_mtime_reuses_reserved_table() {
        let session = Session::new().expect("session");
        let (first, _) = session.reuse_or_reserve("/study/a/adsl.sav", 1, "adsl", |name| {
            placeholder("/study/a/adsl.sav", name)
        });
        let (second, reused) = session.reuse_or_reserve("/study/a/adsl.sav", 1, "adsl", |name| {
            placeholder("/study/a/adsl.sav", name)
        });
        assert!(reused);
        assert_eq!(first.table_name, second.table_name);
        assert_eq!(session.tables.lock().unwrap().len(), 1);
    }

    #[test]
    fn drop_table_removes_data_and_registry() {
        let session = Session::new().expect("session");
        session.reuse_or_reserve("/tmp/ads.sav", 1, "ads", |name| placeholder("/tmp/ads.sav", name));
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
        session.drop_table("ads").expect("drop");
        assert!(session.tables.lock().unwrap().is_empty());
        let q = session.query_conn().expect("query conn");
        assert!(
            q.query_row("SELECT COUNT(*) FROM ads", [], |r| r.get::<_, i64>(0))
                .is_err(),
            "closed dataset must not remain queryable",
        );
        let meta_rows: i64 = q
            .query_row("SELECT COUNT(*) FROM meta_datasets WHERE table_name = 'ads'", [], |r| r.get(0))
            .expect("meta lookup");
        assert_eq!(meta_rows, 0);
    }
}
