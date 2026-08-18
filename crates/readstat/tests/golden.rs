//! Golden / fixture tests for ReadStat parsing.
//!
//! Drop official or generated samples into `tests/fixtures/`:
//! - `sample.sav`, `sample.dta`, `sample.sas7bdat`, `sample.xpt`
//! - placeholders: `xpt_v5.xpt`, `xpt_v8.xpt`, `dta_v113.dta`, `dta_v118.dta`
//!
//! Tests that cannot find a fixture are skipped so CI stays green
//! until samples are checked in.

use std::path::{Path, PathBuf};

use arrow::array::{Array, Float64Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch;
use readstat::{parse_file, BatchSink, DatasetMeta, ParseHooks, ParseOptions};

struct CollectingSink {
    meta: Option<DatasetMeta>,
    batches: Vec<RecordBatch>,
}

impl BatchSink for CollectingSink {
    fn on_metadata(&mut self, meta: &DatasetMeta) -> readstat::Result<()> {
        self.meta = Some(meta.clone());
        Ok(())
    }

    fn on_batch(&mut self, batch: RecordBatch) -> readstat::Result<()> {
        self.batches.push(batch);
        Ok(())
    }
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn parse_if_present(path: &Path) -> Option<(DatasetMeta, Vec<RecordBatch>)> {
    if !path.exists() {
        eprintln!("skip missing fixture {}", path.display());
        return None;
    }
    let mut sink = CollectingSink {
        meta: None,
        batches: Vec::new(),
    };
    let meta = parse_file(path, ParseOptions::default(), ParseHooks { cancel: None, progress: None }, &mut sink)
        .expect("parse fixture");
    Some((meta, sink.batches))
}

fn cell_string(batch: &RecordBatch, row: usize, col: usize) -> Option<String> {
    let arr = batch.column(col);
    if arr.is_null(row) {
        return None;
    }
    if let Some(a) = arr.as_any().downcast_ref::<StringArray>() {
        return Some(a.value(row).to_string());
    }
    if let Some(a) = arr.as_any().downcast_ref::<Int32Array>() {
        return Some(a.value(row).to_string());
    }
    if let Some(a) = arr.as_any().downcast_ref::<Float64Array>() {
        return Some(a.value(row).to_string());
    }
    None
}

#[test]
fn datetime_module_covered_in_unit_tests() {
    let formatted = readstat::format_raw_number(readstat::Origin::Sas, "DATE9.", 0.0);
    assert_eq!(formatted.as_deref(), Some("1960-01-01"));
}

#[test]
fn sav_fixture_if_present() {
    let Some((meta, batches)) = parse_if_present(&fixture("sample.sav")) else {
        return;
    };
    assert!(!meta.variables.is_empty());
    assert!(meta.file.format == readstat::FileFormat::Sav);
    assert!(!batches.is_empty());
}

#[test]
fn dta_fixture_if_present() {
    let Some((meta, batches)) = parse_if_present(&fixture("sample.dta")) else {
        return;
    };
    assert_eq!(meta.file.format, readstat::FileFormat::Dta);
    assert!(!batches.is_empty());
    let _ = cell_string(&batches[0], 0, 0);
}

#[test]
fn sas7bdat_fixture_if_present() {
    let Some((meta, _)) = parse_if_present(&fixture("sample.sas7bdat")) else {
        return;
    };
    assert_eq!(meta.file.format, readstat::FileFormat::Sas7bdat);
}

#[test]
fn xpt_fixture_if_present() {
    let Some((meta, _)) = parse_if_present(&fixture("sample.xpt")) else {
        return;
    };
    assert_eq!(meta.file.format, readstat::FileFormat::Xpt);
    assert!(
        meta.value_labels.is_empty(),
        "XPT does not carry value labels"
    );
}

#[test]
fn xpt_v5_placeholder() {
    let path = fixture("xpt_v5.xpt");
    if !path.exists() {
        eprintln!("TODO: add an XPT v5 golden file at {}", path.display());
        return;
    }
    let (meta, _) = parse_if_present(&path).unwrap();
    assert_eq!(meta.file.format, readstat::FileFormat::Xpt);
}

#[test]
fn xpt_v8_placeholder() {
    let path = fixture("xpt_v8.xpt");
    if !path.exists() {
        eprintln!("TODO: add an XPT v8 golden file at {}", path.display());
        return;
    }
    let (meta, _) = parse_if_present(&path).unwrap();
    assert_eq!(meta.file.format, readstat::FileFormat::Xpt);
}

#[test]
fn dta_v113_placeholder() {
    let path = fixture("dta_v113.dta");
    if !path.exists() {
        eprintln!("TODO: add a Stata v113 golden file at {}", path.display());
        return;
    }
    let (meta, _) = parse_if_present(&path).unwrap();
    assert_eq!(meta.file.format, readstat::FileFormat::Dta);
}

#[test]
fn dta_v118_placeholder() {
    let path = fixture("dta_v118.dta");
    if !path.exists() {
        eprintln!("TODO: add a Stata v118 golden file at {}", path.display());
        return;
    }
    let (meta, _) = parse_if_present(&path).unwrap();
    assert_eq!(meta.file.format, readstat::FileFormat::Dta);
}

#[test]
fn catalog_rejected_as_dataset() {
    let path = fixture("sample.sas7bcat");
    if !path.exists() {
        return;
    }
    let mut sink = CollectingSink {
        meta: None,
        batches: Vec::new(),
    };
    let err = parse_file(
        &path,
        ParseOptions::default(),
        ParseHooks {
            cancel: None,
            progress: None,
        },
        &mut sink,
    )
    .unwrap_err();
    assert!(matches!(err, readstat::Error::CatalogIsNotDataset));
}
