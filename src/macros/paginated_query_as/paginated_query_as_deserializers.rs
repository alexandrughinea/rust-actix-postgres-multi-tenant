use crate::macros::{
    DEFAULT_MAX_FIELD_LENGTH, DEFAULT_MAX_PAGE_SIZE, DEFAULT_MIN_PAGE_SIZE, DEFAULT_PAGE,
};
use serde::{Deserialize, Deserializer};

fn extract_digits_from_strings(val: impl Into<String>) -> String {
    val.into().chars().filter(|c| c.is_ascii_digit()).collect()
}

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
    // First try to deserialize as an Option<String>
    let opt_val = Option::<String>::deserialize(deserializer)?;

    // Handle the Option
    let val = match opt_val {
        None => return Ok(DEFAULT_MIN_PAGE_SIZE),
        Some(s) if s.trim().is_empty() => return Ok(DEFAULT_MIN_PAGE_SIZE),
        Some(s) => s,
    };

    // Extract digits and parse
    let digits: String = extract_digits_from_strings(val);

    // Parse and provide default, clamping between 1 and 100
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
    // First try to deserialize as an Option<String>
    let opt_val = Option::<String>::deserialize(deserializer)?;

    // Handle the Option
    let val = match opt_val {
        None => return Ok(None), // No search if field is missing
        Some(s) if s.trim().is_empty() => return Ok(None), // No search if empty string
        Some(s) => s,
    };

    // Clean and normalize the search string
    let normalized_value = val
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
    let s: Option<String> = Option::deserialize(deserializer)?;

    Ok(s.map(|s| {
        s.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_digits_from_strings() {
        assert_eq!(extract_digits_from_strings("123abc456"), "123456");
        assert_eq!(extract_digits_from_strings("abc"), "");
        assert_eq!(extract_digits_from_strings("1a2b3c"), "123");
        assert_eq!(extract_digits_from_strings(String::from("12.34")), "1234");
        assert_eq!(extract_digits_from_strings("page=5"), "5");
    }
}
