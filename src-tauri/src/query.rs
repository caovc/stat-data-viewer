use std::collections::HashMap;

use arrow::datatypes::DataType;
use duckdb::Connection;
use readstat::{format_raw_number, parse_filter_date_to_raw, FileFormat, Origin, StorageType};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::sqlutil::{quote_ident, quote_string, strip_trailing_semicolon};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortSpec {
    pub column: String,
    pub desc: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterSpec {
    pub column: String,
    pub op: String,
    pub value: Option<String>,
    pub value2: Option<String>,
    #[serde(default)]
    pub values: Option<Vec<String>>,
    #[serde(default)]
    pub include_null: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum FilterNode {
    #[serde(rename = "condition", rename_all = "camelCase")]
    Condition {
        column: String,
        op: String,
        value: Option<String>,
        value2: Option<String>,
        #[serde(default)]
        values: Option<Vec<String>>,
        #[serde(default)]
        include_null: Option<bool>,
    },
    #[serde(rename = "group")]
    Group {
        combinator: String,
        children: Vec<FilterNode>,
    },
}

impl FilterNode {
    pub fn empty_group() -> Self {
        FilterNode::Group {
            combinator: "and".into(),
            children: Vec::new(),
        }
    }

    fn condition(spec: FilterSpec) -> Self {
        FilterNode::Condition {
            column: spec.column,
            op: spec.op,
            value: spec.value,
            value2: spec.value2,
            values: spec.values,
            include_null: spec.include_null,
        }
    }

    fn to_spec(&self) -> Option<FilterSpec> {
        match self {
            FilterNode::Condition {
                column,
                op,
                value,
                value2,
                values,
                include_null,
            } => Some(FilterSpec {
                column: column.clone(),
                op: op.clone(),
                value: value.clone(),
                value2: value2.clone(),
                values: values.clone(),
                include_null: *include_null,
            }),
            FilterNode::Group { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistinctValueOut {
    pub value: Option<String>,
    pub label: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistinctResult {
    pub values: Vec<DistinctValueOut>,
    pub truncated: bool,
    pub empty_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnOut {
    pub name: String,
    pub label: Option<String>,
    pub storage_type: String,
    pub display_format: Option<String>,
    pub origin: String,
    pub is_datetime: bool,
    pub label_set: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResult {
    pub columns: Vec<ColumnOut>,
    pub rows: Vec<Vec<Value>>,
    pub offset: u64,
    pub page_size: u64,
    pub total_rows: u64,
}

#[derive(Clone)]
struct ColMeta {
    name: String,
    label: Option<String>,
    storage: StorageType,
    format: Option<String>,
    origin: Origin,
    label_set: Option<String>,
}

fn load_col_meta(conn: &Connection, table: &str) -> Result<Vec<ColMeta>, String> {
    let origin = dataset_origin(conn, table)?;
    let mut stmt = conn
        .prepare(
            "SELECT name, label, storage_type, display_format, label_set
             FROM meta_variables WHERE table_name = ? ORDER BY var_index",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([table], |r| {
            let storage = match r.get::<_, String>(2)?.as_str() {
                "int32" => StorageType::Int32,
                "string" => StorageType::String,
                _ => StorageType::Float64,
            };
            Ok(ColMeta {
                name: r.get(0)?,
                label: r.get(1)?,
                storage,
                format: r.get(3)?,
                origin,
                label_set: r.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn dataset_origin(conn: &Connection, table: &str) -> Result<Origin, String> {
    let fmt: String = conn
        .query_row(
            "SELECT file_format FROM meta_datasets WHERE table_name = ?",
            [table],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "sas7bdat".into());
    Ok(FileFormat::from_name(&fmt)
        .unwrap_or(FileFormat::Sas7bdat)
        .origin())
}

fn num_literal(col: &ColMeta, raw: &str) -> Result<String, String> {
    let is_dt = col
        .format
        .as_deref()
        .and_then(|fmt| readstat::classify_format(col.origin, fmt))
        .is_some();
    if is_dt {
        if let Some(fmt) = &col.format {
            if let Some(n) = parse_filter_date_to_raw(col.origin, fmt, raw) {
                return Ok(n.to_string());
            }
        }
    }
    raw.parse::<f64>()
        .ok()
        .filter(|n| n.is_finite())
        .map(|n| n.to_string())
        .ok_or_else(|| format!("invalid number: {raw}"))
}

fn value_literal(col: &ColMeta, raw: &str) -> Result<String, String> {
    if col.storage == StorageType::String {
        Ok(quote_string(raw))
    } else {
        num_literal(col, raw)
    }
}

fn empty_clause(ident: &str, col: &ColMeta) -> String {
    if col.storage == StorageType::String {
        format!("({ident} IS NULL OR {ident} = '')")
    } else {
        format!("{ident} IS NULL")
    }
}

fn with_include_null(clause: String, ident: &str, col: &ColMeta, include: bool) -> String {
    if include {
        format!("({clause} OR {})", empty_clause(ident, col))
    } else {
        clause
    }
}

fn compile_in_list(col: &ColMeta, values: &[String]) -> Result<String, String> {
    values
        .iter()
        .map(|value| value_literal(col, value))
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join(", "))
}

fn compile_sorts(sorts: &[SortSpec], cols: &[ColMeta]) -> String {
    let mut seen = Vec::new();
    let mut parts = Vec::new();
    for spec in sorts {
        if seen.iter().any(|name| name == &spec.column) {
            continue;
        }
        if !cols.iter().any(|col| col.name == spec.column) {
            continue;
        }
        seen.push(spec.column.clone());
        parts.push(format!(
            "{} {} NULLS LAST",
            quote_ident(&spec.column),
            if spec.desc { "DESC" } else { "ASC" }
        ));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" ORDER BY {}", parts.join(", "))
    }
}

fn filter_is_active(f: &FilterSpec) -> bool {
    match f.op.as_str() {
        "is_null" | "not_null" | "empty" | "not_empty" => true,
        "in" | "not_in" => {
            f.values.as_ref().is_some_and(|items| !items.is_empty()) || f.include_null.unwrap_or(false)
        }
        "between" => {
            f.value.as_ref().is_some_and(|item| !item.is_empty())
                && f.value2.as_ref().is_some_and(|item| !item.is_empty())
        }
        _ => f.value.as_ref().is_some_and(|item| !item.is_empty()),
    }
}

fn compile_condition(f: &FilterSpec, cols: &[ColMeta]) -> Result<String, String> {
    let col = cols
        .iter()
        .find(|c| c.name == f.column)
        .ok_or_else(|| format!("unknown column {}", f.column))?;
    let ident = quote_ident(&col.name);
    let include_null = f.include_null.unwrap_or(false);
    let clause = match f.op.as_str() {
            "eq" => format!(
                "{ident} = {}",
                value_literal(col, f.value.as_deref().unwrap_or(""))?
            ),
            "ne" => format!(
                "{ident} IS DISTINCT FROM {}",
                value_literal(col, f.value.as_deref().unwrap_or(""))?
            ),
            "contains" => format!(
                "CAST({ident} AS VARCHAR) ILIKE {}",
                quote_string(&format!("%{}%", f.value.clone().unwrap_or_default()))
            ),
            "starts" => format!(
                "CAST({ident} AS VARCHAR) ILIKE {}",
                quote_string(&format!("{}%", f.value.clone().unwrap_or_default()))
            ),
            "ends" => format!(
                "CAST({ident} AS VARCHAR) ILIKE {}",
                quote_string(&format!("%{}", f.value.clone().unwrap_or_default()))
            ),
            "gt" | "gte" | "lt" | "lte" => {
                let op = match f.op.as_str() {
                    "gt" => ">",
                    "gte" => ">=",
                    "lt" => "<",
                    _ => "<=",
                };
                format!(
                    "{ident} {op} {}",
                    value_literal(col, f.value.as_deref().unwrap_or(""))?
                )
            }
            "between" => format!(
                "{ident} BETWEEN {} AND {}",
                value_literal(col, f.value.as_deref().unwrap_or(""))?,
                value_literal(col, f.value2.as_deref().unwrap_or(""))?
            ),
            "in" | "not_in" => {
                let vals = f.values.as_deref().unwrap_or(&[]);
                if vals.is_empty() {
                    if f.op == "in" && include_null {
                        empty_clause(&ident, col)
                    } else if f.op == "not_in" && !include_null {
                        format!("NOT {}", empty_clause(&ident, col))
                    } else if f.op == "in" {
                        "1=0".into()
                    } else {
                        "1=1".into()
                    }
                } else {
                    let list = compile_in_list(col, vals)?;
                    if f.op == "in" {
                        with_include_null(format!("{ident} IN ({list})"), &ident, col, include_null)
                    } else if include_null {
                        format!("({ident} NOT IN ({list}) OR {})", empty_clause(&ident, col))
                    } else {
                        format!(
                            "({ident} IS NOT NULL{} AND {ident} NOT IN ({list}))",
                            if col.storage == StorageType::String {
                                format!(" AND {ident} <> ''")
                            } else {
                                String::new()
                            }
                        )
                    }
                }
            }
            "is_null" | "empty" => empty_clause(&ident, col),
            "not_null" | "not_empty" => format!("NOT {}", empty_clause(&ident, col)),
            other => return Err(format!("unsupported filter op: {other}")),
        };
    let clause = if matches!(f.op.as_str(), "in" | "not_in" | "is_null" | "not_null" | "empty" | "not_empty")
    {
        clause
    } else {
        with_include_null(clause, &ident, col, include_null)
    };
    Ok(clause)
}

fn compile_node(node: &FilterNode, cols: &[ColMeta]) -> Result<Option<String>, String> {
    match node {
        FilterNode::Condition { .. } => {
            let spec = node.to_spec().expect("condition node");
            if !filter_is_active(&spec) {
                return Ok(None);
            }
            Ok(Some(compile_condition(&spec, cols)?))
        }
        FilterNode::Group { combinator, children } => {
            let mut parts = Vec::new();
            for child in children {
                if let Some(sql) = compile_node(child, cols)? {
                    parts.push(sql);
                }
            }
            if parts.is_empty() {
                return Ok(None);
            }
            if parts.len() == 1 {
                return Ok(Some(parts.remove(0)));
            }
            let join = if combinator.eq_ignore_ascii_case("or") {
                " OR "
            } else {
                " AND "
            };
            Ok(Some(format!("({})", parts.join(join))))
        }
    }
}

fn compile_filters(filters: &FilterNode, cols: &[ColMeta]) -> Result<String, String> {
    match compile_node(filters, cols)? {
        Some(sql) => Ok(format!(" WHERE {sql}")),
        None => Ok(String::new()),
    }
}

fn format_value(col: &ColMeta, raw: Value) -> Value {
    let Some(fmt) = col.format.as_deref() else {
        return raw;
    };
    let num = match &raw {
        Value::Number(n) => n.as_f64(),
        Value::Null => return raw,
        _ => return raw,
    };
    let Some(n) = num else {
        return raw;
    };
    match format_raw_number(col.origin, fmt, n) {
        Some(s) => Value::String(s),
        None => raw,
    }
}

pub fn query_page(
    conn: &Connection,
    table: &str,
    offset: u64,
    page_size: u64,
    sorts: &[SortSpec],
    filters: &FilterNode,
    hidden: &[String],
) -> Result<PageResult, String> {
    let all = load_col_meta(conn, table)?;
    let where_sql = compile_filters(filters, &all)?;
    let order = compile_sorts(sorts, &all);
    let visible: Vec<ColMeta> = all
        .into_iter()
        .filter(|c| !hidden.iter().any(|h| h == &c.name))
        .collect();
    if visible.is_empty() {
        return Ok(PageResult {
            columns: vec![],
            rows: vec![],
            offset,
            page_size,
            total_rows: 0,
        });
    }
    let select = visible
        .iter()
        .map(|c| quote_ident(&c.name))
        .collect::<Vec<_>>()
        .join(", ");
    let from = quote_ident(table);
    let count_sql = format!("SELECT COUNT(*) FROM {from}{where_sql}");
    let total: i64 = conn
        .query_row(&count_sql, [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let data_sql = format!(
        "SELECT {select} FROM {from}{where_sql}{order} LIMIT {page_size} OFFSET {offset}"
    );
    let mut stmt = conn.prepare(&data_sql).map_err(|e| e.to_string())?;
    let mut rows_iter = stmt.query([]).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    while let Some(row) = rows_iter.next().map_err(|e| e.to_string())? {
        let mut cells = Vec::with_capacity(visible.len());
        for (i, col) in visible.iter().enumerate() {
            let raw = cell_value(row, i, col.storage)?;
            cells.push(format_value(col, raw));
        }
        rows.push(cells);
    }
    Ok(PageResult {
        columns: visible.iter().map(col_out).collect(),
        rows,
        offset,
        page_size,
        total_rows: total as u64,
    })
}

fn col_out(c: &ColMeta) -> ColumnOut {
    ColumnOut {
        name: c.name.clone(),
        label: c.label.clone(),
        storage_type: c.storage.as_str().into(),
        display_format: c.format.clone(),
        origin: c.origin.as_str().into(),
        is_datetime: c
            .format
            .as_deref()
            .and_then(|f| readstat::classify_format(c.origin, f))
            .is_some(),
        label_set: c.label_set.clone(),
    }
}

fn cell_value(row: &duckdb::Row<'_>, i: usize, storage: StorageType) -> Result<Value, String> {
    match storage {
        StorageType::String => {
            let v: Option<String> = row.get(i).map_err(|e| e.to_string())?;
            Ok(v.map(Value::String).unwrap_or(Value::Null))
        }
        StorageType::Int32 => {
            let v: Option<i32> = row.get(i).map_err(|e| e.to_string())?;
            Ok(v.map(|n| json!(n)).unwrap_or(Value::Null))
        }
        StorageType::Float64 => {
            let v: Option<f64> = row.get(i).map_err(|e| e.to_string())?;
            Ok(v.map(|n| json!(n)).unwrap_or(Value::Null))
        }
    }
}

pub fn run_sql_page(
    conn: &Connection,
    sql: &str,
    offset: u64,
    page_size: u64,
) -> Result<PageResult, String> {
    let inner = strip_trailing_semicolon(sql);
    if inner.is_empty() {
        return Err("empty SQL".into());
    }
    let count_sql = format!("SELECT COUNT(*) FROM ({inner}) AS _sdv_q");
    let total: i64 = conn
        .query_row(&count_sql, [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let data_sql = format!("SELECT * FROM ({inner}) AS _sdv_q LIMIT {page_size} OFFSET {offset}");
    let mut stmt = conn.prepare(&data_sql).map_err(|e| e.to_string())?;
    let mut rows_iter = stmt.query([]).map_err(|e| e.to_string())?;
    let columns = sql_result_columns(conn, &rows_iter)?;
    let mut rows = Vec::new();
    while let Some(row) = rows_iter.next().map_err(|e| e.to_string())? {
        let mut cells = Vec::with_capacity(columns.len());
        for (i, col) in columns.iter().enumerate() {
            cells.push(sql_cell_for(row, i, col)?);
        }
        rows.push(cells);
    }
    Ok(PageResult {
        columns,
        rows,
        offset,
        page_size,
        total_rows: total as u64,
    })
}

pub(crate) fn describe_sql_columns(conn: &Connection, sql: &str) -> Result<Vec<ColumnOut>, String> {
    let inner = strip_trailing_semicolon(sql);
    if inner.is_empty() {
        return Err("empty SQL".into());
    }
    let data_sql = format!("SELECT * FROM ({inner}) AS _sdv_q LIMIT 0");
    let mut stmt = conn.prepare(&data_sql).map_err(|e| e.to_string())?;
    let rows = stmt.query([]).map_err(|e| e.to_string())?;
    sql_result_columns(conn, &rows)
}

fn sql_result_columns(conn: &Connection, rows: &duckdb::Rows<'_>) -> Result<Vec<ColumnOut>, String> {
    let stmt = rows
        .as_ref()
        .ok_or_else(|| "query produced no statement schema".to_string())?;
    let schema = stmt.schema();
    let meta = load_unique_var_meta(conn);
    Ok(schema
        .fields()
        .iter()
        .map(|field| {
            if let Some(col) = meta.get(field.name()) {
                return col_out(col);
            }
            let (storage_type, is_datetime) = duck_col_type(field.data_type());
            ColumnOut {
                name: field.name().clone(),
                label: None,
                storage_type,
                display_format: None,
                origin: "duckdb".into(),
                is_datetime,
                label_set: None,
            }
        })
        .collect())
}

fn load_unique_var_meta(conn: &Connection) -> HashMap<String, ColMeta> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT v.name, v.label, v.storage_type, v.display_format, v.label_set, d.file_format
         FROM meta_variables v
         LEFT JOIN meta_datasets d ON d.table_name = v.table_name",
    ) else {
        return HashMap::new();
    };
    let Ok(iter) = stmt.query_map([], |r| {
        let fmt: Option<String> = r.get(5)?;
        let origin = FileFormat::from_name(fmt.as_deref().unwrap_or("sas7bdat"))
            .unwrap_or(FileFormat::Sas7bdat)
            .origin();
        let storage = match r.get::<_, String>(2)?.as_str() {
            "int32" => StorageType::Int32,
            "string" => StorageType::String,
            _ => StorageType::Float64,
        };
        Ok(ColMeta {
            name: r.get(0)?,
            label: r.get(1)?,
            storage,
            format: r.get(3)?,
            origin,
            label_set: r.get(4)?,
        })
    }) else {
        return HashMap::new();
    };
    let mut grouped: HashMap<String, Vec<ColMeta>> = HashMap::new();
    for row in iter.flatten() {
        grouped.entry(row.name.clone()).or_default().push(row);
    }
    grouped
        .into_iter()
        .filter_map(|(name, mut cols)| (cols.len() == 1).then(|| (name, cols.pop().unwrap())))
        .collect()
}

fn duck_col_type(dt: &DataType) -> (String, bool) {
    match dt {
        DataType::Boolean
        | DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32 => ("int32".into(), false),
        DataType::Int64 | DataType::UInt64 => ("int64".into(), false),
        DataType::Float16 | DataType::Float32 | DataType::Float64 => ("float64".into(), false),
        DataType::Decimal32(_, _)
        | DataType::Decimal64(_, _)
        | DataType::Decimal128(_, _)
        | DataType::Decimal256(_, _) => ("float64".into(), false),
        DataType::Date32
        | DataType::Date64
        | DataType::Time32(_)
        | DataType::Time64(_)
        | DataType::Timestamp(_, _)
        | DataType::Duration(_) => ("float64".into(), true),
        _ => ("string".into(), false),
    }
}

fn sql_cell_for(row: &duckdb::Row<'_>, i: usize, col: &ColumnOut) -> Result<Value, String> {
    if col.is_datetime {
        if let Ok(v) = row.get::<_, Option<String>>(i) {
            return Ok(v.map(Value::String).unwrap_or(Value::Null));
        }
    }
    match col.storage_type.as_str() {
        "string" => {
            if let Ok(v) = row.get::<_, Option<String>>(i) {
                return Ok(v.map(Value::String).unwrap_or(Value::Null));
            }
        }
        "int32" => {
            if let Ok(v) = row.get::<_, Option<i32>>(i) {
                return Ok(v.map(|n| json!(n)).unwrap_or(Value::Null));
            }
            if let Ok(v) = row.get::<_, Option<i64>>(i) {
                return Ok(v.map(|n| json!(n)).unwrap_or(Value::Null));
            }
        }
        "int64" => {
            if let Ok(v) = row.get::<_, Option<i64>>(i) {
                return Ok(v.map(|n| json!(n)).unwrap_or(Value::Null));
            }
        }
        "float64" => {
            if let Ok(v) = row.get::<_, Option<f64>>(i) {
                return Ok(v.map(|n| json!(n)).unwrap_or(Value::Null));
            }
        }
        _ => {}
    }
    sql_cell(row, i)
}

fn number_to_filter_value(n: f64) -> String {
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

fn distinct_pair(col: &ColMeta, raw: Value) -> Option<(String, String)> {
    match raw {
        Value::Null => None,
        Value::String(s) if s.is_empty() => None,
        Value::String(s) => Some((s.clone(), s)),
        Value::Number(n) => {
            let value = n.as_f64().map(number_to_filter_value).unwrap_or_else(|| n.to_string());
            let label = match format_value(col, Value::Number(n)) {
                Value::String(s) => s,
                Value::Number(formatted) => formatted.to_string(),
                other => other.to_string(),
            };
            Some((value, label))
        }
        other => {
            let s = other.to_string();
            Some((s.clone(), s))
        }
    }
}

pub fn column_distinct(
    conn: &Connection,
    table: &str,
    column: &str,
    limit: u64,
) -> Result<DistinctResult, String> {
    let cols = load_col_meta(conn, table)?;
    let col = cols
        .iter()
        .find(|c| c.name == column)
        .ok_or_else(|| format!("unknown column {column}"))?;
    let ident = quote_ident(&col.name);
    let from = quote_ident(table);
    let cap = limit.clamp(1, 2000);
    let sql = format!(
        "SELECT {ident} AS v, COUNT(*) AS n FROM {from} GROUP BY 1 ORDER BY (v IS NULL) DESC, n DESC, v LIMIT {}",
        cap + 1
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    let mut values = Vec::new();
    let mut empty_count = 0i64;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let raw = cell_value(row, 0, col.storage)?;
        let count: i64 = row.get(1).map_err(|e| e.to_string())?;
        if raw.is_null() || matches!(&raw, Value::String(s) if s.is_empty()) {
            empty_count += count;
            continue;
        }
        if let Some((value, label)) = distinct_pair(col, raw) {
            values.push(DistinctValueOut {
                value: Some(value),
                label: Some(label),
                count,
            });
        }
    }
    let truncated = values.len() as u64 > cap;
    if truncated {
        values.truncate(cap as usize);
    }
    Ok(DistinctResult {
        values,
        truncated,
        empty_count,
    })
}

fn sql_cell(row: &duckdb::Row<'_>, i: usize) -> Result<Value, String> {
    if let Ok(v) = row.get::<_, Option<i64>>(i) {
        return Ok(v.map(|n| json!(n)).unwrap_or(Value::Null));
    }
    if let Ok(v) = row.get::<_, Option<f64>>(i) {
        return Ok(v.map(|n| json!(n)).unwrap_or(Value::Null));
    }
    if let Ok(v) = row.get::<_, Option<String>>(i) {
        return Ok(v.map(Value::String).unwrap_or(Value::Null));
    }
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Session;

    #[test]
    fn query_page_reads_rows_written_on_session() {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (id INTEGER, name VARCHAR);
                 INSERT INTO ads VALUES (1, 'a'), (2, 'b');
                 INSERT INTO meta_datasets VALUES ('ads', '/tmp/ads.sav', 0, 'sav', NULL, NULL, NULL, NULL, 2, 2, NULL, true);
                 INSERT INTO meta_variables VALUES ('ads', 0, 'id', NULL, 'int32', NULL, NULL, NULL, NULL, '[]', NULL);
                 INSERT INTO meta_variables VALUES ('ads', 1, 'name', NULL, 'string', NULL, NULL, NULL, NULL, '[]', NULL);",
            )
            .expect("seed");
        }
        let conn = session.query_conn().expect("query conn");
        let page = query_page(&conn, "ads", 0, 200, &[], &FilterNode::empty_group(), &[]).expect("page");
        assert_eq!(page.total_rows, 2);
        assert_eq!(page.columns.len(), 2);
        assert_eq!(page.rows.len(), 2);
        assert_eq!(page.rows[0][1], Value::String("a".into()));
    }

    #[test]
    fn run_sql_page_reads_select() {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (id INTEGER, name VARCHAR);
                 INSERT INTO ads VALUES (1, 'a'), (2, 'b');",
            )
            .expect("seed");
        }
        let conn = session.query_conn().expect("query conn");
        let page = run_sql_page(&conn, "SELECT * FROM ads", 0, 200).expect("sql page");
        assert_eq!(page.total_rows, 2);
        assert_eq!(
            page.columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            ["id", "name"]
        );
        assert_eq!(page.columns[0].storage_type, "int32");
        assert_eq!(page.columns[1].storage_type, "string");
        assert_eq!(page.rows.len(), 2);
        assert_eq!(page.rows[0][0], json!(1));
        assert_eq!(page.rows[0][1], Value::String("a".into()));
    }

    #[test]
    fn run_sql_page_keeps_numeric_and_date_types() {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (id INTEGER, score DOUBLE, name VARCHAR, dt DATE);
                 INSERT INTO ads VALUES (1, 1.5, 'a', DATE '2020-01-01');",
            )
            .expect("seed");
        }
        let conn = session.query_conn().expect("query conn");
        let page = run_sql_page(&conn, "SELECT * FROM ads", 0, 200).expect("sql page");
        assert_eq!(
            page.columns
                .iter()
                .map(|c| (c.name.as_str(), c.storage_type.as_str(), c.is_datetime))
                .collect::<Vec<_>>(),
            [
                ("id", "int32", false),
                ("score", "float64", false),
                ("name", "string", false),
                ("dt", "float64", true),
            ]
        );
    }

    #[test]
    fn run_sql_page_overlays_unique_import_meta() {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (visit DOUBLE, name VARCHAR);
                 INSERT INTO ads VALUES (21915, 'a');
                 INSERT INTO meta_datasets VALUES ('ads', '/tmp/ads.sav', 0, 'sav', NULL, NULL, NULL, NULL, 1, 2, NULL, true);
                 INSERT INTO meta_variables VALUES ('ads', 0, 'visit', 'Visit date', 'float64', 'DATE9.', NULL, NULL, NULL, '[]', NULL);
                 INSERT INTO meta_variables VALUES ('ads', 1, 'name', 'Subject', 'string', NULL, NULL, NULL, NULL, '[]', NULL);",
            )
            .expect("seed");
        }
        let conn = session.query_conn().expect("query conn");
        let page = run_sql_page(&conn, "SELECT * FROM ads", 0, 200).expect("sql page");
        assert_eq!(page.columns[0].storage_type, "float64");
        assert!(page.columns[0].is_datetime, "imported date format should stay datetime");
        assert_eq!(page.columns[0].label.as_deref(), Some("Visit date"));
        assert_eq!(page.columns[1].storage_type, "string");
        assert_eq!(page.columns[1].label.as_deref(), Some("Subject"));
    }

    fn filter(
        column: &str,
        op: &str,
        value: Option<&str>,
        value2: Option<&str>,
        values: Option<Vec<&str>>,
        include_null: Option<bool>,
    ) -> FilterSpec {
        FilterSpec {
            column: column.into(),
            op: op.into(),
            value: value.map(str::to_string),
            value2: value2.map(str::to_string),
            values: values.map(|items| items.into_iter().map(str::to_string).collect()),
            include_null,
        }
    }

    fn and_all(items: &[FilterSpec]) -> FilterNode {
        FilterNode::Group {
            combinator: "and".into(),
            children: items.iter().cloned().map(FilterNode::condition).collect(),
        }
    }

    fn group(combinator: &str, children: Vec<FilterNode>) -> FilterNode {
        FilterNode::Group {
            combinator: combinator.into(),
            children,
        }
    }

    #[test]
    fn query_page_filters_by_type_and_value_list() {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (id INTEGER, score DOUBLE, name VARCHAR);
                 INSERT INTO ads VALUES (1, 1.5, 'alice'), (2, 2.5, 'bob'), (3, NULL, NULL), (4, 4.0, '');
                 INSERT INTO meta_datasets VALUES ('ads', '/tmp/ads.sav', 0, 'sav', NULL, NULL, NULL, NULL, 4, 3, NULL, true);
                 INSERT INTO meta_variables VALUES ('ads', 0, 'id', NULL, 'int32', NULL, NULL, NULL, NULL, '[]', NULL);
                 INSERT INTO meta_variables VALUES ('ads', 1, 'score', NULL, 'float64', NULL, NULL, NULL, NULL, '[]', NULL);
                 INSERT INTO meta_variables VALUES ('ads', 2, 'name', NULL, 'string', NULL, NULL, NULL, NULL, '[]', NULL);",
            )
            .expect("seed");
        }
        let conn = session.query_conn().expect("query conn");

        let page = query_page(
            &conn,
            "ads",
            0,
            200,
            &[],
            &and_all(&[filter("score", "gt", Some("2"), None, None, None)]),
            &[],
        )
        .expect("gt");
        assert_eq!(page.total_rows, 2);

        let page = query_page(
            &conn,
            "ads",
            0,
            200,
            &[],
            &and_all(&[filter("name", "contains", Some("li"), None, None, None)]),
            &[],
        )
        .expect("contains");
        assert_eq!(page.total_rows, 1);
        assert_eq!(page.rows[0][2], Value::String("alice".into()));

        let page = query_page(
            &conn,
            "ads",
            0,
            200,
            &[],
            &and_all(&[filter("name", "ends", Some("ob"), None, None, None)]),
            &[],
        )
        .expect("ends");
        assert_eq!(page.total_rows, 1);

        let page = query_page(
            &conn,
            "ads",
            0,
            200,
            &[],
            &and_all(&[filter("id", "in", None, None, Some(vec!["1", "3"]), None)]),
            &[],
        )
        .expect("in");
        assert_eq!(page.total_rows, 2);

        let page = query_page(
            &conn,
            "ads",
            0,
            200,
            &[],
            &and_all(&[filter("score", "gt", Some("2"), None, None, Some(true))]),
            &[],
        )
        .expect("include null");
        assert_eq!(page.total_rows, 3);

        let page = query_page(
            &conn,
            "ads",
            0,
            200,
            &[],
            &and_all(&[filter("name", "in", None, None, Some(vec!["alice"]), Some(true))]),
            &[],
        )
        .expect("in + empty");
        assert_eq!(page.total_rows, 3);

        let page = query_page(
            &conn,
            "ads",
            0,
            200,
            &[],
            &and_all(&[filter("name", "is_null", None, None, None, None)]),
            &[],
        )
        .expect("empty");
        assert_eq!(page.total_rows, 2);
    }

    #[test]
    fn query_page_filters_nested_and_or() {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (id INTEGER, name VARCHAR);
                 INSERT INTO ads VALUES (1, 'a'), (2, 'a'), (1, 'b'), (2, 'b');
                 INSERT INTO meta_datasets VALUES ('ads', '/tmp/ads.sav', 0, 'sav', NULL, NULL, NULL, NULL, 4, 2, NULL, true);
                 INSERT INTO meta_variables VALUES ('ads', 0, 'id', NULL, 'int32', NULL, NULL, NULL, NULL, '[]', NULL);
                 INSERT INTO meta_variables VALUES ('ads', 1, 'name', NULL, 'string', NULL, NULL, NULL, NULL, '[]', NULL);",
            )
            .expect("seed");
        }
        let conn = session.query_conn().expect("query conn");

        let page = query_page(
            &conn,
            "ads",
            0,
            200,
            &[],
            &group(
                "or",
                vec![
                    group(
                        "and",
                        vec![
                            FilterNode::condition(filter("id", "eq", Some("1"), None, None, None)),
                            FilterNode::condition(filter("name", "eq", Some("a"), None, None, None)),
                        ],
                    ),
                    group(
                        "and",
                        vec![
                            FilterNode::condition(filter("id", "eq", Some("2"), None, None, None)),
                            FilterNode::condition(filter("name", "eq", Some("b"), None, None, None)),
                        ],
                    ),
                ],
            ),
            &[],
        )
        .expect("nested");
        assert_eq!(page.total_rows, 2);

        let page = query_page(
            &conn,
            "ads",
            0,
            200,
            &[],
            &group(
                "or",
                vec![
                    FilterNode::condition(filter("id", "eq", Some("1"), None, None, None)),
                    FilterNode::condition(filter("name", "eq", Some("b"), None, None, None)),
                ],
            ),
            &[],
        )
        .expect("or");
        assert_eq!(page.total_rows, 3);
    }

    #[test]
    fn filter_node_deserializes_nested_json() {
        let node: FilterNode = serde_json::from_str(
            r#"{
                "type": "group",
                "combinator": "or",
                "children": [
                    {"type": "condition", "column": "id", "op": "eq", "value": "1"},
                    {
                        "type": "group",
                        "combinator": "and",
                        "children": [
                            {"type": "condition", "column": "name", "op": "contains", "value": "a"}
                        ]
                    }
                ]
            }"#,
        )
        .expect("json");
        match node {
            FilterNode::Group { combinator, children } => {
                assert_eq!(combinator, "or");
                assert_eq!(children.len(), 2);
            }
            FilterNode::Condition { .. } => panic!("expected group"),
        }
    }

    fn sort(column: &str, desc: bool) -> SortSpec {
        SortSpec {
            column: column.into(),
            desc,
        }
    }

    #[test]
    fn query_page_sorts_by_multiple_columns() {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (id INTEGER, name VARCHAR);
                 INSERT INTO ads VALUES (2, 'b'), (1, 'b'), (1, 'a');
                 INSERT INTO meta_datasets VALUES ('ads', '/tmp/ads.sav', 0, 'sav', NULL, NULL, NULL, NULL, 3, 2, NULL, true);
                 INSERT INTO meta_variables VALUES ('ads', 0, 'id', NULL, 'int32', NULL, NULL, NULL, NULL, '[]', NULL);
                 INSERT INTO meta_variables VALUES ('ads', 1, 'name', NULL, 'string', NULL, NULL, NULL, NULL, '[]', NULL);",
            )
            .expect("seed");
        }
        let conn = session.query_conn().expect("query conn");
        let page = query_page(
            &conn,
            "ads",
            0,
            200,
            &[sort("name", false), sort("id", true)],
            &FilterNode::empty_group(),
            &[],
        )
        .expect("sorts");
        assert_eq!(page.total_rows, 3);
        assert_eq!(page.rows[0][1], Value::String("a".into()));
        assert_eq!(page.rows[0][0], json!(1));
        assert_eq!(page.rows[1][1], Value::String("b".into()));
        assert_eq!(page.rows[1][0], json!(2));
        assert_eq!(page.rows[2][1], Value::String("b".into()));
        assert_eq!(page.rows[2][0], json!(1));
    }

    #[test]
    fn column_distinct_counts_empty_separately() {
        let session = Session::new().expect("session");
        {
            let conn = session.write_conn();
            conn.execute_batch(
                "CREATE TABLE ads (name VARCHAR);
                 INSERT INTO ads VALUES ('a'), ('a'), ('b'), (NULL), ('');
                 INSERT INTO meta_datasets VALUES ('ads', '/tmp/ads.sav', 0, 'sav', NULL, NULL, NULL, NULL, 5, 1, NULL, true);
                 INSERT INTO meta_variables VALUES ('ads', 0, 'name', NULL, 'string', NULL, NULL, NULL, NULL, '[]', NULL);",
            )
            .expect("seed");
        }
        let conn = session.query_conn().expect("query conn");
        let result = column_distinct(&conn, "ads", "name", 500).expect("distinct");
        assert_eq!(result.empty_count, 2);
        assert_eq!(result.values.len(), 2);
        assert_eq!(result.values[0].value.as_deref(), Some("a"));
        assert_eq!(result.values[0].count, 2);
        assert!(!result.truncated);
    }
}
