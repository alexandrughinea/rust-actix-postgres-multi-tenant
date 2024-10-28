use chrono::{DateTime, Utc};
use crate::macros::paginated_query_as::{QueryParams, SortDirection};
use crate::macros::{DEFAULT_MIN_PAGE_SIZE, DEFAULT_PAGE, DEFAULT_SEARCH_COLUMNS};
use serde::Serialize;
use serde_json::Value;
use sqlx::postgres::PgArguments;
use sqlx::Arguments;
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
    let parts: Vec<&str> = identifier.split('.').collect();
    parts
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
        _ => ""
    }
}

pub fn build_pg_arguments<T>(params: &QueryParams) -> (Vec<String>, PgArguments)
where
    T: Default + Serialize,
{
    let valid_columns_from_struct = get_struct_field_names::<T>();
    let mut arguments = PgArguments::default();
    let mut conditions = Vec::new();

    // Add only valid filter conditions
    for (key, value) in &params.filters {
        if valid_columns_from_struct.contains(key) {
            if let Some(value) = value {
                let table_column = quote_identifier(key);
                let table_column_value = get_pg_type_cast(value);
                let next_argument = arguments.len() + 1;


                conditions.push(format!("{} = ${}{}", table_column, next_argument, table_column_value));
                let _ = arguments.add(value);
            }
        } else {
            tracing::warn!(column = %key, "Skipping invalid filter column");
        }
    }

    // Add only valid search conditions
    if let Some(search) = &params.search.search {
        if let Some(columns) = &params.search.search_columns {
            let valid_search_columns: Vec<&String> = columns
                .iter()
                .filter(|column| valid_columns_from_struct.contains(*column))
                .collect();

            if !valid_search_columns.is_empty() && !search.trim().is_empty() {
                if !search.is_empty() {
                    let pattern = format!("%{}%", search);
                    let is_uuid = Uuid::try_parse(search).is_ok();
                    let next_argument = arguments.len() + 1;

                    let search_conditions: Vec<String> = valid_search_columns
                        .iter()
                        .map(|column| {
                            let table_column = quote_identifier(column);
                            
                            if is_uuid {
                                format!("CAST({} AS text) ILIKE ${}", table_column, next_argument)
                            } else {
                                format!("LOWER({}) LIKE LOWER(${})", table_column, next_argument)
                            }
                        })
                        .collect();

                    if !search_conditions.is_empty() {
                        conditions.push(format!("({})", search_conditions.join(" OR ")));
                        let _ = arguments.add(pattern);
                    }
                }
            } else if !columns.is_empty() {
                tracing::warn!("No valid search columns found among: {:?}", columns);
            }
        }
    }

    // Add date range conditions (fixed columns)
    if let Some(after) = params.date_range.created_after {
        conditions.push(format!("created_at >= ${}", arguments.len() + 1));
        let _ = arguments.add(after);
    }

    if let Some(before) = params.date_range.created_before {
        conditions.push(format!("created_at <= ${}", arguments.len() + 1));
        let _ = arguments.add(before);
    }

    (conditions, arguments)
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
