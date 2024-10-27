/// List of potentially dangerous prefixes and full names
const UNSAFE_PG_PATTERNS: &[&str] = &[
    // System schemas and tables
    "pg_",
    "information_schema.",
    // System columns
    "oid",
    "tableoid",
    "xmin",
    "xmax",
    "cmin",
    "cmax",
    "ctid",
    // Other sensitive prefixes
    "pg_catalog",
    "pg_toast",
    "pg_temp",
    "pg_internal",
];

/// Validates column names to prevent SQL injection and access to sensitive data.
pub fn is_safe_column_name(string_like: impl Into<String>) -> bool {
    let value: String = string_like.into();
    // Basic safety checks
    if value.is_empty()
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        || value.contains("..")
        || value.starts_with('.')
        || value.ends_with('.')
    {
        return false;
    }

    // Check against unsafe patterns
    let lowercase = value.to_lowercase();

    !UNSAFE_PG_PATTERNS
        .iter()
        .any(|pattern| lowercase.contains(*pattern))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_columns() {
        // Valid
        assert!(is_safe_column_name("user_id"));
        assert!(is_safe_column_name("email123"));
        assert!(is_safe_column_name("public.users.id"));
        assert!(is_safe_column_name("my_table_name"));

        // Invalid - SQL injection attempts
        assert!(!is_safe_column_name(""));
        assert!(!is_safe_column_name("drop--table"));
        assert!(!is_safe_column_name("users;drop"));
        assert!(!is_safe_column_name("table*"));
        assert!(!is_safe_column_name(".hidden"));
        assert!(!is_safe_column_name("public..users"));
    }

    #[test]
    fn test_system_tables() {
        // These should all be blocked
        assert!(!is_safe_column_name("pg_catalog.pg_tables"));
        assert!(!is_safe_column_name("information_schema.tables"));
        assert!(!is_safe_column_name("pg_statistic"));
        assert!(!is_safe_column_name("pg_user"));
    }

    #[test]
    fn test_system_columns() {
        // These should all be blocked
        assert!(!is_safe_column_name("ctid"));
        assert!(!is_safe_column_name("oid"));
        assert!(!is_safe_column_name("tableoid"));
        assert!(!is_safe_column_name("xmin"));
        assert!(!is_safe_column_name("xmax"));
        assert!(!is_safe_column_name("cmin"));
        assert!(!is_safe_column_name("cmax"));
    }

    #[test]
    fn test_case_insensitive() {
        // These should all be blocked regardless of case
        assert!(!is_safe_column_name("PG_CATALOG.pg_tables"));
        assert!(!is_safe_column_name("INFORMATION_SCHEMA.tables"));
        assert!(!is_safe_column_name("OID"));
        assert!(!is_safe_column_name("CTID"));
    }
}
