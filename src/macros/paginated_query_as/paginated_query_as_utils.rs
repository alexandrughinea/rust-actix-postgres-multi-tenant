use crate::macros::paginated_query_as::{QueryParams, SortDirection};
use serde::{Deserialize, Deserializer};
use sqlx::postgres::PgArguments;
use sqlx::Arguments;
use uuid::Uuid;

/// Deserializes a page number with proper default handling
pub fn deserialize_page<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    // First try to deserialize as an Option<String>
    let opt_val = Option::<String>::deserialize(deserializer)?;

    // Handle the Option
    let val = match opt_val {
        None => return Ok(1),                           // Default if field is missing
        Some(s) if s.trim().is_empty() => return Ok(1), // Default if empty string
        Some(s) => s,
    };

    // Extract digits and parse
    let digits: String = val.chars().filter(|c| c.is_ascii_digit()).collect();

    // Parse and provide default
    digits
        .parse::<i64>()
        .map(|n| if n < 1 { 1 } else { n })
        .or(Ok(1))
}

/// Deserializes a page size with proper default handling
pub fn deserialize_page_size<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    // First try to deserialize as an Option<String>
    let opt_val = Option::<String>::deserialize(deserializer)?;

    // Handle the Option
    let val = match opt_val {
        None => return Ok(10),                           // Default if field is missing
        Some(s) if s.trim().is_empty() => return Ok(10), // Default if empty string
        Some(s) => s,
    };

    // Extract digits and parse
    let digits: String = val.chars().filter(|c| c.is_ascii_digit()).collect();

    // Parse and provide default, clamping between 1 and 100
    digits.parse::<i64>().map(|n| n.clamp(1, 100)).or(Ok(10))
}

/// Deserializes a search string with proper sanitization and default handling
pub fn deserialize_search<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // First try to deserialize as an Option<String>
    let opt_val = Option::<String>::deserialize(deserializer)?;

    // Handle the Option
    let val = match opt_val {
        None => return Ok(None), // No search if field is missing
        Some(s) if s.trim().is_empty() => return Ok(None), // No search if empty string
        Some(s) => s,
    };

    // Clean and normalize the search string
    let cleaned = val
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(100)
        .collect::<String>();

    if cleaned.is_empty() {
        Ok(None)
    } else {
        Ok(Some(cleaned))
    }
}

pub fn default_page() -> i64 {
    1
}

pub fn default_page_size() -> i64 {
    10
}

pub fn default_search_columns() -> Option<Vec<String>> {
    Some(vec!["name".to_string()])
}

pub fn default_sort_field() -> String {
    "created_at".to_string()
}

pub fn default_sort_direction() -> SortDirection {
    SortDirection::Descending
}

pub fn build_pg_arguments(params: &QueryParams) -> (Vec<String>, PgArguments) {
    let mut arguments = PgArguments::default();
    let mut conditions = Vec::new();

    for (key, value) in &params.filters {
        if let Some(value) = value {
            let quoted_key = quote_identifier(key);
            conditions.push(format!("{} = ${}", quoted_key, arguments.len() + 1));
            let _ = arguments.add(value);
        }
    }

    if let Some(search) = &params.search.search {
        if let Some(columns) = &params.search.search_columns {
            if !columns.is_empty() && !search.trim().is_empty() {
                let search_term_sanitized = search
                    .trim()
                    .chars()
                    .filter(|&c| {
                        // Allow only alphanumeric characters and basic punctuation
                        c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_'
                    })
                    .collect::<String>();

                if !search_term_sanitized.is_empty() {
                    let pattern = format!("%{}%", search_term_sanitized);
                    let is_uuid = Uuid::try_parse(&search_term_sanitized).is_ok();

                    let search_conditions: Vec<String> = columns
                        .iter()
                        .filter_map(|column| {
                            // @todo Also validate column name to prevent SQL injection
                            let quoted_column = quote_identifier(column);

                            Some(if is_uuid {
                                format!(
                                    "CAST({} AS text) ILIKE ${}",
                                    quoted_column,
                                    arguments.len() + 1
                                )
                            } else {
                                format!(
                                    "LOWER({}) LIKE LOWER(${})",
                                    quoted_column,
                                    arguments.len() + 1
                                )
                            })
                        })
                        .collect();

                    if !search_conditions.is_empty() {
                        conditions.push(format!("({})", search_conditions.join(" OR ")));
                        let _ = arguments.add(pattern);
                    }
                }
            }
        }
    }
    // Add date range
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

pub fn quote_identifier(identifier: &str) -> String {
    let parts: Vec<&str> = identifier.split('.').collect();
    parts
        .iter()
        .map(|part| format!("\"{}\"", part.replace("\"", "\"\"")))
        .collect::<Vec<_>>()
        .join(".")
}
