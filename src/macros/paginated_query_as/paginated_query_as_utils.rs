use crate::macros::paginated_query_as::SortDirection;
use crate::macros::{DEFAULT_MIN_PAGE_SIZE, DEFAULT_PAGE, DEFAULT_SEARCH_COLUMNS};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

pub fn default_page() -> i64 {
    DEFAULT_PAGE
}

pub fn default_page_size() -> i64 {
    DEFAULT_MIN_PAGE_SIZE
}

pub fn default_search_columns() -> Option<Vec<String>> {
    Some(
        DEFAULT_SEARCH_COLUMNS
            .iter()
            .map(|&s| s.to_string())
            .collect(),
    )
}

pub fn default_sort_field() -> String {
    "created_at".to_string()
}

pub fn default_sort_direction() -> SortDirection {
    SortDirection::Descending
}

pub fn quote_identifier(identifier: &str) -> String {
    identifier
        .split('.')
        .collect::<Vec<&str>>()
        .iter()
        .map(|part| format!("\"{}\"", part.replace("\"", "\"\"")))
        .collect::<Vec<_>>()
        .join(".")
}

pub fn get_struct_field_names<T>() -> Vec<String>
where
    T: Default + Serialize,
{
    let default_value = T::default();
    let json_value = serde_json::to_value(default_value).unwrap();

    if let Value::Object(map) = json_value {
        map.keys().cloned().collect()
    } else {
        vec![]
    }
}

pub fn get_pg_type_cast<T: ToString>(value: &T) -> &'static str {
    let str_value = value.to_string();
    match str_value.to_lowercase().as_str() {
        // Booleans
        v if v == "true" || v == "false" => "::boolean",

        // Numbers
        v if v.parse::<i64>().is_ok() => "::bigint",
        v if v.parse::<f64>().is_ok() => "::double precision",

        // UUIDs
        v if Uuid::parse_str(v).is_ok() => "::uuid",

        // JSON
        v if v.starts_with('{') || v.starts_with('[') => "::jsonb",

        // Dates/Timestamps
        v if v.parse::<DateTime<Utc>>().is_ok() => "::timestamp with time zone",

        // Default - no type cast
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Default, Serialize)]
    struct TestStruct {
        field1: String,
        field2: i32,
        #[serde(skip)]
        #[allow(dead_code)]
        field3: bool,
    }

    #[test]
    fn test_get_fields() {
        let fields = get_struct_field_names::<TestStruct>();
        assert!(fields.contains(&"field1".to_string()));
        assert!(fields.contains(&"field2".to_string()));
        assert!(!fields.contains(&"field3".to_string()));
    }
}
