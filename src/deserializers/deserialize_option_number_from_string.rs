use serde::{de, Deserialize, Deserializer};
use std::str::FromStr;

pub fn deserialize_option_number_from_string<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + std::fmt::Debug,
    T::Err: std::fmt::Display,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) if s.is_empty() || s.to_lowercase() == "null" || s.to_lowercase() == "none" => {
            Ok(None)
        }
        None => Ok(None),
        Some(value) => match value.parse::<T>() {
            Ok(num) => Ok(Some(num)),
            Err(e) => Err(de::Error::custom(format!(
                "Failed to parse '{}' as the correct numeric type: {}",
                value, e
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestStruct {
        #[serde(deserialize_with = "deserialize_option_number_from_string")]
        int_field: Option<i32>,

        #[serde(deserialize_with = "deserialize_option_number_from_string")]
        #[serde(default)]
        float_field: Option<f64>,
    }

    #[test]
    fn test_number_deserialization() {
        // Test valid numbers
        let data = json!({
            "int_field": "42",
            "float_field": "3.14"
        });
        let result: TestStruct = serde_json::from_value(data).unwrap();
        assert_eq!(result.int_field, Some(42));
        assert_eq!(result.float_field, Some(3.14));

        // Test null values (various forms)
        let null_cases = vec![
            json!({
                "int_field": "null",
                "float_field": "NULL"
            }),
            json!({
                "int_field": "none",
                "float_field": "NONE"
            }),
            json!({
                "int_field": "",
                "float_field": ""
            }),
            json!({
                "int_field": null,
                "float_field": null
            }),
        ];

        for case in null_cases {
            let result: TestStruct = serde_json::from_value(case).unwrap();
            assert_eq!(result.int_field, None);
            assert_eq!(result.float_field, None);
        }
    }

    #[test]
    fn test_invalid_numbers() {
        let data = json!({
            "int_field": "not_a_number",
            "float_field": "3.14"
        });
        let result = serde_json::from_value::<TestStruct>(data);
        assert!(result.is_err());

        // Test float in integer field
        let data = json!({
            "int_field": "3.14",
            "float_field": "3.14"
        });
        let result = serde_json::from_value::<TestStruct>(data);
        assert!(result.is_err());

        // Test invalid float
        let data = json!({
            "int_field": "42",
            "float_field": "not_a_float"
        });
        let result = serde_json::from_value::<TestStruct>(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_boundary_values() {
        // Test max/min values
        let data = json!({
            "int_field": i32::MAX.to_string(),
            "float_field": f64::MAX.to_string()
        });
        let result: TestStruct = serde_json::from_value(data).unwrap();
        assert_eq!(result.int_field, Some(i32::MAX));
        assert_eq!(result.float_field, Some(f64::MAX));

        // Test negative values
        let data = json!({
            "int_field": "-42",
            "float_field": "-3.14"
        });
        let result: TestStruct = serde_json::from_value(data).unwrap();
        assert_eq!(result.int_field, Some(-42));
        assert_eq!(result.float_field, Some(-3.14));
    }

    #[test]
    fn test_scientific_notation() {
        let data = json!({
            "int_field": "100", // Changed from "1e2"
            "float_field": "1.23e-4"
        });
        let result: TestStruct = serde_json::from_value(data).unwrap();
        assert_eq!(result.int_field, Some(100));
        assert_eq!(result.float_field, Some(0.000123));
    }

    #[test]
    fn test_partial_fields() {
        let data = json!({
            "int_field": "42"
            // float_field will be None due to #[serde(default)]
        });
        let result: TestStruct = serde_json::from_value(data).unwrap();
        assert_eq!(result.int_field, Some(42));
        assert_eq!(result.float_field, None);
    }
}
