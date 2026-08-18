use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    Sas7bdat,
    Xpt,
    Sav,
    Por,
    Dta,
    Sas7bcat,
}

impl FileFormat {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_string_lossy().to_ascii_lowercase();
        match ext.as_str() {
            "sas7bdat" => Some(Self::Sas7bdat),
            "xpt" | "xport" => Some(Self::Xpt),
            "sav" | "zsav" => Some(Self::Sav),
            "por" => Some(Self::Por),
            "dta" => Some(Self::Dta),
            "sas7bcat" => Some(Self::Sas7bcat),
            _ => None,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "sas7bdat" | "sas" => Some(Self::Sas7bdat),
            "xpt" | "xport" => Some(Self::Xpt),
            "sav" | "zsav" | "spss" => Some(Self::Sav),
            "por" => Some(Self::Por),
            "dta" | "stata" => Some(Self::Dta),
            "sas7bcat" | "catalog" => Some(Self::Sas7bcat),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sas7bdat => "sas7bdat",
            Self::Xpt => "xpt",
            Self::Sav => "sav",
            Self::Por => "por",
            Self::Dta => "dta",
            Self::Sas7bcat => "sas7bcat",
        }
    }

    pub fn origin(self) -> Origin {
        match self {
            Self::Sas7bdat | Self::Xpt | Self::Sas7bcat => Origin::Sas,
            Self::Sav | Self::Por => Origin::Spss,
            Self::Dta => Origin::Stata,
        }
    }

    pub fn is_dataset(self) -> bool {
        !matches!(self, Self::Sas7bcat)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    Sas,
    Spss,
    Stata,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sas => "sas",
            Self::Spss => "spss",
            Self::Stata => "stata",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    String,
    Int32,
    Float64,
}

impl StorageType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Int32 => "int32",
            Self::Float64 => "float64",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingRule {
    pub lo: Option<f64>,
    pub hi: Option<f64>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableMeta {
    pub index: i32,
    pub name: String,
    pub label: Option<String>,
    pub storage_type: StorageType,
    pub display_format: Option<String>,
    pub measure: Option<String>,
    pub display_width: Option<i32>,
    pub decimals: Option<i32>,
    pub missing_rules: Vec<MissingRule>,
    pub label_set: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueLabel {
    pub label_set: String,
    pub num_value: Option<f64>,
    pub str_value: Option<String>,
    pub tag: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub path: PathBuf,
    pub format: FileFormat,
    pub row_count: Option<i64>,
    pub var_count: i32,
    pub file_label: Option<String>,
    pub file_encoding: Option<String>,
    pub table_name: Option<String>,
    pub format_version: Option<i32>,
    pub catalog_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMeta {
    pub file: FileMeta,
    pub variables: Vec<VariableMeta>,
    pub value_labels: Vec<ValueLabel>,
}

#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub encoding: Option<String>,
    pub format: Option<FileFormat>,
    pub catalog_path: Option<PathBuf>,
    pub batch_size: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            encoding: None,
            format: None,
            catalog_path: None,
            batch_size: 2_000,
        }
    }
}

pub fn sanitize_table_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("dataset");
    let mut out = String::new();
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        "dataset".into()
    } else if out.as_bytes()[0].is_ascii_digit() {
        format!("t_{out}")
    } else {
        out
    }
}

pub fn find_default_catalog(data_path: &Path) -> Option<PathBuf> {
    let dir = data_path.parent()?;
    let stem = data_path.file_stem()?.to_string_lossy();
    let candidates = [
        dir.join(format!("{stem}.sas7bcat")),
        dir.join(format!("{stem}.SAS7BCAT")),
        dir.join("formats.sas7bcat"),
        dir.join("FORMATS.sas7bcat"),
        dir.join("format.sas7bcat"),
    ];
    candidates.into_iter().find(|p| p.exists())
}
