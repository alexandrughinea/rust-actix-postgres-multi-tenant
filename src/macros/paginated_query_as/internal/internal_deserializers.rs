use crate::macros::paginated_query_as::internal::{
    extract_digits_from_strings, DEFAULT_MAX_FIELD_LENGTH, DEFAULT_MAX_PAGE_SIZE,
    DEFAULT_MIN_PAGE_SIZE, DEFAULT_PAGE,
};
use serde::{Deserialize, Deserializer};

/// Deserializes a page number with proper default handling
pub fn deserialize_page<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    // First try to deserialize as an Option<String>
    let opt_val = Option::<String>::deserialize(deserializer)?;

    // Handle the Option
    let val = match opt_val {
        None => return Ok(DEFAULT_PAGE),
        Some(s) if s.trim().is_empty() => return Ok(DEFAULT_PAGE),
        Some(s) => s,
    };

    // Extract digits and parse
    let digits: String = extract_digits_from_strings(val);

    // Parse and provide default
    digits
        .parse::<i64>()
        .map(|n| if n < DEFAULT_PAGE { DEFAULT_PAGE } else { n })
        .or(Ok(DEFAULT_PAGE))
}

/// Deserializes a page size with proper default handling
pub fn deserialize_page_size<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    let value_with_fallbacks = match value {
        None => return Ok(DEFAULT_MIN_PAGE_SIZE),
        Some(s) if s.trim().is_empty() => return Ok(DEFAULT_MIN_PAGE_SIZE),
        Some(s) => s,
    };

    // Extract digits and parse
    let digits: String = extract_digits_from_strings(value_with_fallbacks);

    // Parse and provide default, clamping between {default_min} and {default_max}
    digits
        .parse::<i64>()
        .map(|n| n.clamp(DEFAULT_MIN_PAGE_SIZE, DEFAULT_MAX_PAGE_SIZE))
        .or(Ok(DEFAULT_MIN_PAGE_SIZE))
}

/// Deserializes a search string with proper sanitization and default handling
pub fn deserialize_search<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    let value_with_fallbacks = match value {
        None => return Ok(None), // No search if field is missing
        Some(s) if s.trim().is_empty() => return Ok(None), // No search if empty string
        Some(s) => s,
    };

    // Clean and normalize the search string
    let normalized_value = value_with_fallbacks
        .trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(DEFAULT_MAX_FIELD_LENGTH as usize)
        .collect::<String>();

    if normalized_value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(normalized_value))
    }
}

pub fn deserialize_search_columns<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;

    Ok(value.map(|s| {
        s.split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .collect()
    }))
}
