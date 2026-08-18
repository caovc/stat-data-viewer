pub fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

pub fn quote_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub fn strip_trailing_semicolon(sql: &str) -> &str {
    sql.trim().trim_end_matches(';').trim()
}
