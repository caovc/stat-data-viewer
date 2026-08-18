//! Safe ReadStat wrapper: C callbacks → Arrow `RecordBatch` stream.

mod datetime;
mod error;
mod parser;
mod types;

pub use datetime::{classify_format, format_raw_number, parse_filter_date_to_raw, DateKind};
pub use error::{Error, Result};
pub use parser::{parse_catalog, parse_file, BatchSink, ParseHooks};
pub use types::{
    find_default_catalog, sanitize_table_name, DatasetMeta, FileFormat, FileMeta, MissingRule,
    Origin, ParseOptions, StorageType, ValueLabel, VariableMeta,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_name_sanitizes() {
        assert_eq!(
            sanitize_table_name(std::path::Path::new("/tmp/ADSL.sas7bdat")),
            "adsl"
        );
        assert_eq!(
            sanitize_table_name(std::path::Path::new("123 data.sav")),
            "t_123_data"
        );
    }
}
