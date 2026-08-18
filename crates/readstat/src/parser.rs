use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use arrow::array::{Array, ArrayBuilder, ArrayRef, Float64Builder, Int32Builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use readstat_sys as sys;

use crate::error::{Error, Result};
use crate::types::{
    DatasetMeta, FileFormat, FileMeta, MissingRule, ParseOptions, StorageType, ValueLabel,
    VariableMeta,
};

const HANDLER_OK: c_int = 0;
const HANDLER_ABORT: c_int = 1;

pub trait BatchSink {
    fn on_metadata(&mut self, meta: &DatasetMeta) -> Result<()>;
    fn on_batch(&mut self, batch: RecordBatch) -> Result<()>;
}

pub struct ParseHooks<'a> {
    pub cancel: Option<Arc<AtomicBool>>,
    pub progress: Option<&'a dyn Fn(f64)>,
}

struct ColumnAcc {
    storage: StorageType,
    i32s: Option<Int32Builder>,
    f64s: Option<Float64Builder>,
    strs: Option<StringBuilder>,
}

impl ColumnAcc {
    fn new(storage: StorageType, capacity: usize) -> Self {
        match storage {
            StorageType::Int32 => Self {
                storage,
                i32s: Some(Int32Builder::with_capacity(capacity)),
                f64s: None,
                strs: None,
            },
            StorageType::Float64 => Self {
                storage,
                i32s: None,
                f64s: Some(Float64Builder::with_capacity(capacity)),
                strs: None,
            },
            StorageType::String => Self {
                storage,
                i32s: None,
                f64s: None,
                strs: Some(StringBuilder::with_capacity(capacity, capacity * 8)),
            },
        }
    }

    fn append_null(&mut self) {
        match self.storage {
            StorageType::Int32 => self.i32s.as_mut().unwrap().append_null(),
            StorageType::Float64 => self.f64s.as_mut().unwrap().append_null(),
            StorageType::String => self.strs.as_mut().unwrap().append_null(),
        }
    }

    fn append_i32(&mut self, v: i32) {
        match self.storage {
            StorageType::Int32 => self.i32s.as_mut().unwrap().append_value(v),
            StorageType::Float64 => self.f64s.as_mut().unwrap().append_value(v as f64),
            StorageType::String => self.strs.as_mut().unwrap().append_value(v.to_string()),
        }
    }

    fn append_f64(&mut self, v: f64) {
        match self.storage {
            StorageType::Int32 => {
                if v.fract() == 0.0 && v >= i32::MIN as f64 && v <= i32::MAX as f64 {
                    self.i32s.as_mut().unwrap().append_value(v as i32);
                } else {
                    self.i32s.as_mut().unwrap().append_null();
                }
            }
            StorageType::Float64 => self.f64s.as_mut().unwrap().append_value(v),
            StorageType::String => self.strs.as_mut().unwrap().append_value(v.to_string()),
        }
    }

    fn append_str(&mut self, v: &str) {
        match self.storage {
            StorageType::String => self.strs.as_mut().unwrap().append_value(v),
            StorageType::Int32 => self.i32s.as_mut().unwrap().append_null(),
            StorageType::Float64 => self.f64s.as_mut().unwrap().append_null(),
        }
    }

    fn finish(&mut self) -> ArrayRef {
        match self.storage {
            StorageType::Int32 => {
                let mut b = self.i32s.take().unwrap();
                let arr = b.finish();
                self.i32s = Some(Int32Builder::with_capacity(arr.len().max(16)));
                std::sync::Arc::new(arr)
            }
            StorageType::Float64 => {
                let mut b = self.f64s.take().unwrap();
                let arr = b.finish();
                self.f64s = Some(Float64Builder::with_capacity(arr.len().max(16)));
                std::sync::Arc::new(arr)
            }
            StorageType::String => {
                let mut b = self.strs.take().unwrap();
                let arr = b.finish();
                self.strs = Some(StringBuilder::with_capacity(arr.len().max(16), arr.len().max(16) * 8));
                std::sync::Arc::new(arr)
            }
        }
    }

    fn len(&self) -> usize {
        match self.storage {
            StorageType::Int32 => self.i32s.as_ref().unwrap().len(),
            StorageType::Float64 => self.f64s.as_ref().unwrap().len(),
            StorageType::String => self.strs.as_ref().unwrap().len(),
        }
    }
}

struct ParseCtx<'a> {
    cancel: Option<Arc<AtomicBool>>,
    progress: Option<&'a dyn Fn(f64)>,
    sink: &'a mut dyn BatchSink,
    format: FileFormat,
    path: std::path::PathBuf,
    catalog_path: Option<std::path::PathBuf>,
    batch_size: usize,
    file_meta: Option<FileMeta>,
    variables: Vec<VariableMeta>,
    value_labels: Vec<ValueLabel>,
    columns: Vec<ColumnAcc>,
    schema: Option<std::sync::Arc<Schema>>,
    current_row: i32,
    pending: Vec<PendingCell>,
    rows_in_batch: usize,
    metadata_emitted: bool,
    abort: bool,
    error: Option<String>,
}

#[derive(Clone)]
enum PendingCell {
    Unset,
    Null,
    I32(i32),
    F64(f64),
    Str(String),
}

impl<'a> ParseCtx<'a> {
    fn cancelled(&self) -> bool {
        self.cancel
            .as_ref()
            .is_some_and(|c| c.load(Ordering::Relaxed))
    }

    fn ensure_row(&mut self, obs: i32) -> Result<()> {
        if self.variables.is_empty() {
            return Ok(());
        }
        if self.pending.is_empty() {
            self.pending = vec![PendingCell::Unset; self.variables.len()];
            self.current_row = obs;
        }
        if obs != self.current_row {
            self.flush_row()?;
            self.current_row = obs;
            self.pending.fill(PendingCell::Unset);
        }
        Ok(())
    }

    fn flush_row(&mut self) -> Result<()> {
        if self.pending.is_empty() || self.columns.is_empty() {
            return Ok(());
        }
        if !self.metadata_emitted {
            self.emit_metadata()?;
        }
        for (idx, cell) in self.pending.iter().enumerate() {
            match cell {
                PendingCell::Unset | PendingCell::Null => self.columns[idx].append_null(),
                PendingCell::I32(v) => self.columns[idx].append_i32(*v),
                PendingCell::F64(v) => self.columns[idx].append_f64(*v),
                PendingCell::Str(v) => self.columns[idx].append_str(v),
            }
        }
        self.rows_in_batch += 1;
        if self.rows_in_batch >= self.batch_size {
            self.emit_batch()?;
        }
        Ok(())
    }

    fn emit_metadata(&mut self) -> Result<()> {
        if self.metadata_emitted {
            return Ok(());
        }
        let file = self.file_meta.clone().unwrap_or(FileMeta {
            path: self.path.clone(),
            format: self.format,
            row_count: None,
            var_count: self.variables.len() as i32,
            file_label: None,
            file_encoding: None,
            table_name: None,
            format_version: None,
            catalog_path: self.catalog_path.clone(),
        });
        let meta = DatasetMeta {
            file,
            variables: self.variables.clone(),
            value_labels: self.value_labels.clone(),
        };
        self.sink.on_metadata(&meta)?;
        self.metadata_emitted = true;
        Ok(())
    }

    fn emit_batch(&mut self) -> Result<()> {
        if self.columns.is_empty() || self.rows_in_batch == 0 {
            return Ok(());
        }
        let schema = self.schema.clone().ok_or_else(|| Error::msg("missing schema"))?;
        let arrays: Vec<ArrayRef> = self.columns.iter_mut().map(|c| c.finish()).collect();
        let batch = RecordBatch::try_new(schema, arrays).map_err(|e| Error::Arrow(e.to_string()))?;
        self.sink.on_batch(batch)?;
        self.rows_in_batch = 0;
        Ok(())
    }

    fn finish(&mut self) -> Result<u64> {
        if !self.pending.is_empty() && self.pending.iter().any(|c| !matches!(c, PendingCell::Unset))
        {
            self.flush_row()?;
        }
        if !self.metadata_emitted {
            self.emit_metadata()?;
        }
        if self.rows_in_batch > 0 {
            self.emit_batch()?;
        }
        Ok(self.columns.first().map(|c| c.len() as u64).unwrap_or(0))
    }
}

fn cstr<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }.to_str().ok().filter(|s| !s.is_empty())
}

fn storage_of(ty: sys::readstat_type_t) -> StorageType {
    match ty {
        sys::readstat_type_e_READSTAT_TYPE_STRING | sys::readstat_type_e_READSTAT_TYPE_STRING_REF => {
            StorageType::String
        }
        sys::readstat_type_e_READSTAT_TYPE_INT8
        | sys::readstat_type_e_READSTAT_TYPE_INT16
        | sys::readstat_type_e_READSTAT_TYPE_INT32 => StorageType::Int32,
        _ => StorageType::Float64,
    }
}

fn measure_name(m: sys::readstat_measure_t) -> Option<String> {
    match m {
        sys::readstat_measure_e_READSTAT_MEASURE_NOMINAL => Some("nominal".into()),
        sys::readstat_measure_e_READSTAT_MEASURE_ORDINAL => Some("ordinal".into()),
        sys::readstat_measure_e_READSTAT_MEASURE_SCALE => Some("scale".into()),
        _ => None,
    }
}

fn extract_cell(value: sys::readstat_value_t) -> PendingCell {
    unsafe {
        if sys::readstat_value_is_system_missing(value) != 0
            || sys::readstat_value_is_tagged_missing(value) != 0
        {
            return PendingCell::Null;
        }
        match sys::readstat_value_type(value) {
            sys::readstat_type_e_READSTAT_TYPE_STRING | sys::readstat_type_e_READSTAT_TYPE_STRING_REF => {
                match cstr(sys::readstat_string_value(value)) {
                    Some(s) => PendingCell::Str(s.to_string()),
                    None => PendingCell::Null,
                }
            }
            sys::readstat_type_e_READSTAT_TYPE_INT8 => {
                PendingCell::I32(sys::readstat_int8_value(value) as i32)
            }
            sys::readstat_type_e_READSTAT_TYPE_INT16 => {
                PendingCell::I32(sys::readstat_int16_value(value) as i32)
            }
            sys::readstat_type_e_READSTAT_TYPE_INT32 => PendingCell::I32(sys::readstat_int32_value(value)),
            sys::readstat_type_e_READSTAT_TYPE_FLOAT => {
                PendingCell::F64(sys::readstat_float_value(value) as f64)
            }
            _ => PendingCell::F64(sys::readstat_double_value(value)),
        }
    }
}

extern "C" fn handle_metadata(meta: *mut sys::readstat_metadata_t, ctx: *mut c_void) -> c_int {
    catch(ctx, |c| {
        if meta.is_null() {
            return Ok(());
        }
        unsafe {
            let row_count = sys::readstat_get_row_count(meta);
            c.file_meta = Some(FileMeta {
                path: c.path.clone(),
                format: c.format,
                row_count: if row_count >= 0 { Some(row_count as i64) } else { None },
                var_count: sys::readstat_get_var_count(meta),
                file_label: cstr(sys::readstat_get_file_label(meta)).map(str::to_string),
                file_encoding: cstr(sys::readstat_get_file_encoding(meta)).map(str::to_string),
                table_name: cstr(sys::readstat_get_table_name(meta)).map(str::to_string),
                format_version: Some(sys::readstat_get_file_format_version(meta)),
                catalog_path: c.catalog_path.clone(),
            });
        }
        Ok(())
    })
}

extern "C" fn handle_variable(
    index: c_int,
    variable: *mut sys::readstat_variable_t,
    val_labels: *const c_char,
    ctx: *mut c_void,
) -> c_int {
    catch(ctx, |c| {
        if variable.is_null() {
            return Ok(());
        }
        unsafe {
            let ty = sys::readstat_variable_get_type(variable);
            let storage = storage_of(ty);
            let mut rules = Vec::new();
            let n = sys::readstat_variable_get_missing_ranges_count(variable);
            for i in 0..n {
                let lo = sys::readstat_variable_get_missing_range_lo(variable, i);
                let hi = sys::readstat_variable_get_missing_range_hi(variable, i);
                if sys::readstat_value_type_class(lo) == sys::readstat_type_class_e_READSTAT_TYPE_CLASS_STRING
                {
                    rules.push(MissingRule {
                        lo: None,
                        hi: None,
                        text: cstr(sys::readstat_string_value(lo)).map(str::to_string),
                    });
                } else {
                    rules.push(MissingRule {
                        lo: Some(sys::readstat_double_value(lo)),
                        hi: Some(sys::readstat_double_value(hi)),
                        text: None,
                    });
                }
            }
            c.variables.push(VariableMeta {
                index,
                name: cstr(sys::readstat_variable_get_name(variable))
                    .unwrap_or("var")
                    .to_string(),
                label: cstr(sys::readstat_variable_get_label(variable)).map(str::to_string),
                storage_type: storage,
                display_format: cstr(sys::readstat_variable_get_format(variable)).map(str::to_string),
                measure: measure_name(sys::readstat_variable_get_measure(variable)),
                display_width: Some(sys::readstat_variable_get_display_width(variable)),
                decimals: None,
                missing_rules: rules,
                label_set: cstr(val_labels).map(str::to_string),
            });
        }
        Ok(())
    })
}

extern "C" fn handle_value_label(
    val_labels: *const c_char,
    value: sys::readstat_value_t,
    label: *const c_char,
    ctx: *mut c_void,
) -> c_int {
    catch(ctx, |c| {
        let set = cstr(val_labels).unwrap_or("labels").to_string();
        let label = cstr(label).unwrap_or("").to_string();
        unsafe {
            if sys::readstat_value_is_tagged_missing(value) != 0 {
                let tag = sys::readstat_value_tag(value);
                c.value_labels.push(ValueLabel {
                    label_set: set,
                    num_value: None,
                    str_value: None,
                    tag: Some((tag as u8 as char).to_string()),
                    label,
                });
            } else if sys::readstat_value_type_class(value)
                == sys::readstat_type_class_e_READSTAT_TYPE_CLASS_STRING
            {
                c.value_labels.push(ValueLabel {
                    label_set: set,
                    num_value: None,
                    str_value: cstr(sys::readstat_string_value(value)).map(str::to_string),
                    tag: None,
                    label,
                });
            } else {
                c.value_labels.push(ValueLabel {
                    label_set: set,
                    num_value: Some(sys::readstat_double_value(value)),
                    str_value: None,
                    tag: None,
                    label,
                });
            }
        }
        Ok(())
    })
}

extern "C" fn handle_value(
    obs_index: c_int,
    variable: *mut sys::readstat_variable_t,
    value: sys::readstat_value_t,
    ctx: *mut c_void,
) -> c_int {
    catch(ctx, |c| {
        if c.columns.is_empty() {
            c.init_columns();
        }
        c.ensure_row(obs_index)?;
        let idx = if variable.is_null() {
            0
        } else {
            unsafe { sys::readstat_variable_get_index(variable) as usize }
        };
        if idx < c.pending.len() {
            c.pending[idx] = extract_cell(value);
        }
        Ok(())
    })
}

extern "C" fn handle_progress(progress: f64, ctx: *mut c_void) -> c_int {
    catch(ctx, |c| {
        if let Some(cb) = c.progress {
            cb(progress);
        }
        Ok(())
    })
}

extern "C" fn handle_error(message: *const c_char, ctx: *mut c_void) {
    let _ = catch(ctx, |c| {
        if let Some(m) = cstr(message) {
            c.error = Some(m.to_string());
        }
        Ok(())
    });
}

impl ParseCtx<'_> {
    fn init_columns(&mut self) {
        if !self.columns.is_empty() {
            return;
        }
        let fields: Vec<Field> = self
            .variables
            .iter()
            .map(|v| {
                let dt = match v.storage_type {
                    StorageType::String => DataType::Utf8,
                    StorageType::Int32 => DataType::Int32,
                    StorageType::Float64 => DataType::Float64,
                };
                Field::new(&v.name, dt, true)
            })
            .collect();
        self.schema = Some(std::sync::Arc::new(Schema::new(fields)));
        self.columns = self
            .variables
            .iter()
            .map(|v| ColumnAcc::new(v.storage_type, self.batch_size))
            .collect();
        self.pending = vec![PendingCell::Unset; self.variables.len()];
    }
}

fn catch(ctx: *mut c_void, f: impl FnOnce(&mut ParseCtx<'_>) -> Result<()>) -> c_int {
    if ctx.is_null() {
        return HANDLER_ABORT;
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let ctx = unsafe { &mut *(ctx as *mut ParseCtx<'_>) };
        if ctx.cancelled() {
            ctx.abort = true;
            return HANDLER_ABORT;
        }
        match f(ctx) {
            Ok(()) => HANDLER_OK,
            Err(e) => {
                ctx.error = Some(e.to_string());
                ctx.abort = true;
                HANDLER_ABORT
            }
        }
    }));
    match result {
        Ok(code) => code,
        Err(_) => HANDLER_ABORT,
    }
}

struct ParserGuard {
    ptr: *mut sys::readstat_parser_t,
}

impl ParserGuard {
    fn new() -> Result<Self> {
        let ptr = unsafe { sys::readstat_parser_init() };
        if ptr.is_null() {
            return Err(Error::msg("readstat_parser_init failed"));
        }
        Ok(Self { ptr })
    }
}

impl Drop for ParserGuard {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { sys::readstat_parser_free(self.ptr) };
        }
    }
}

fn apply_common(
    parser: *mut sys::readstat_parser_t,
    encoding: Option<&CString>,
) -> Result<()> {
    unsafe {
        check(sys::readstat_set_error_handler(parser, Some(handle_error)))?;
        check(sys::readstat_set_progress_handler(parser, Some(handle_progress)))?;
        check(sys::readstat_set_metadata_handler(parser, Some(handle_metadata)))?;
        check(sys::readstat_set_variable_handler(parser, Some(handle_variable)))?;
        check(sys::readstat_set_value_handler(parser, Some(handle_value)))?;
        check(sys::readstat_set_value_label_handler(parser, Some(handle_value_label)))?;
        // output_encoding already defaults to the "UTF-8" literal inside ReadStat.
        if let Some(enc) = encoding {
            check(sys::readstat_set_file_character_encoding(parser, enc.as_ptr()))?;
        }
    }
    Ok(())
}

fn check(err: sys::readstat_error_t) -> Result<()> {
    if err == sys::readstat_error_e_READSTAT_OK {
        Ok(())
    } else {
        let msg = unsafe { cstr(sys::readstat_error_message(err)) }
            .unwrap_or("unknown ReadStat error")
            .to_string();
        Err(Error::ReadStat(msg))
    }
}

fn parse_dispatch(
    parser: *mut sys::readstat_parser_t,
    path: &CString,
    format: FileFormat,
    ctx: *mut c_void,
) -> Result<()> {
    let err = unsafe {
        match format {
            FileFormat::Sas7bdat => sys::readstat_parse_sas7bdat(parser, path.as_ptr(), ctx),
            FileFormat::Xpt => sys::readstat_parse_xport(parser, path.as_ptr(), ctx),
            FileFormat::Sav => sys::readstat_parse_sav(parser, path.as_ptr(), ctx),
            FileFormat::Por => sys::readstat_parse_por(parser, path.as_ptr(), ctx),
            FileFormat::Dta => sys::readstat_parse_dta(parser, path.as_ptr(), ctx),
            FileFormat::Sas7bcat => sys::readstat_parse_sas7bcat(parser, path.as_ptr(), ctx),
        }
    };
    if err == sys::readstat_error_e_READSTAT_ERROR_USER_ABORT {
        return Err(Error::Cancelled);
    }
    check(err)
}

pub fn parse_catalog(path: &Path, encoding: Option<&str>) -> Result<Vec<ValueLabel>> {
    let mut sink = NopSink;
    let mut ctx = ParseCtx {
        cancel: None,
        progress: None,
        sink: &mut sink,
        format: FileFormat::Sas7bcat,
        path: path.to_path_buf(),
        catalog_path: Some(path.to_path_buf()),
        batch_size: 1,
        file_meta: None,
        variables: Vec::new(),
        value_labels: Vec::new(),
        columns: Vec::new(),
        schema: None,
        current_row: 0,
        pending: Vec::new(),
        rows_in_batch: 0,
        metadata_emitted: true,
        abort: false,
        error: None,
    };
    let parser = ParserGuard::new()?;
    let c_path = CString::new(path.to_string_lossy().as_bytes()).map_err(|_| Error::InvalidPath)?;
    let enc = encoding.and_then(|e| CString::new(e).ok());
    apply_common(parser.ptr, enc.as_ref())?;
    parse_dispatch(parser.ptr, &c_path, FileFormat::Sas7bcat, &mut ctx as *mut _ as *mut c_void)?;
    Ok(ctx.value_labels)
}

struct NopSink;
impl BatchSink for NopSink {
    fn on_metadata(&mut self, _meta: &DatasetMeta) -> Result<()> {
        Ok(())
    }
    fn on_batch(&mut self, _batch: RecordBatch) -> Result<()> {
        Ok(())
    }
}

pub fn parse_file(
    path: &Path,
    opts: ParseOptions,
    hooks: ParseHooks<'_>,
    sink: &mut dyn BatchSink,
) -> Result<DatasetMeta> {
    let format = opts
        .format
        .or_else(|| FileFormat::from_path(path))
        .ok_or_else(|| Error::UnsupportedFormat(path.display().to_string()))?;
    if format == FileFormat::Sas7bcat {
        return Err(Error::CatalogIsNotDataset);
    }

    let catalog_path = opts
        .catalog_path
        .clone()
        .or_else(|| {
            if format == FileFormat::Sas7bdat {
                crate::types::find_default_catalog(path)
            } else {
                None
            }
        });

    let mut catalog_labels = Vec::new();
    if let Some(cat) = &catalog_path {
        catalog_labels = parse_catalog(cat, opts.encoding.as_deref()).unwrap_or_default();
    }

    let mut ctx = ParseCtx {
        cancel: hooks.cancel.clone(),
        progress: hooks.progress,
        sink,
        format,
        path: path.to_path_buf(),
        catalog_path: catalog_path.clone(),
        batch_size: opts.batch_size.max(256),
        file_meta: None,
        variables: Vec::new(),
        value_labels: catalog_labels,
        columns: Vec::new(),
        schema: None,
        current_row: 0,
        pending: Vec::new(),
        rows_in_batch: 0,
        metadata_emitted: false,
        abort: false,
        error: None,
    };

    let parser = ParserGuard::new()?;
    let c_path = CString::new(path.to_string_lossy().as_bytes()).map_err(|_| Error::InvalidPath)?;
    let enc = opts.encoding.as_ref().and_then(|e| CString::new(e.as_str()).ok());
    apply_common(parser.ptr, enc.as_ref())?;
    let parse_res = parse_dispatch(parser.ptr, &c_path, format, &mut ctx as *mut _ as *mut c_void);
    if ctx.abort {
        if hooks.cancel.as_ref().is_some_and(|c| c.load(Ordering::Relaxed)) {
            return Err(Error::Cancelled);
        }
        if let Some(e) = ctx.error.take() {
            return Err(Error::ReadStat(e));
        }
    }
    parse_res?;
    ctx.finish()?;

    if let Some(file) = ctx.file_meta.as_mut() {
        file.catalog_path = catalog_path.clone();
    }

    attach_catalog_label_sets(&mut ctx.variables, &ctx.value_labels);

    Ok(DatasetMeta {
        file: ctx.file_meta.unwrap_or(FileMeta {
            path: path.to_path_buf(),
            format,
            row_count: None,
            var_count: ctx.variables.len() as i32,
            file_label: None,
            file_encoding: None,
            table_name: None,
            format_version: None,
            catalog_path,
        }),
        variables: ctx.variables,
        value_labels: ctx.value_labels,
    })
}

/// SAS catalogs name label sets after formats. If a variable has a format but no
/// embedded label set, attach the catalog set with the same name.
fn attach_catalog_label_sets(variables: &mut [VariableMeta], labels: &[ValueLabel]) {
    let sets: HashMap<&str, ()> = labels.iter().map(|l| (l.label_set.as_str(), ())).collect();
    for var in variables.iter_mut() {
        if var.label_set.is_some() {
            continue;
        }
        if let Some(fmt) = &var.display_format {
            let key = fmt.trim_end_matches('.').trim();
            if sets.contains_key(key) {
                var.label_set = Some(key.to_string());
            }
        }
    }
}
