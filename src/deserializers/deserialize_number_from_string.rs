use serde::{de, Deserialize, Deserializer};
use std::str::FromStr;

pub fn deserialize_number_from_string<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + std::fmt::Debug,
    T::Err: std::fmt::Display,
{
    let s: String = String::deserialize(deserializer)?;
    s.parse::<T>().map_err(|e| {
        de::Error::custom(format!(
            "Failed to parse '{}' as the correct numeric type: {}",
            s, e
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestStruct {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        required_int: i32,

        #[serde(deserialize_with = "deserialize_number_from_string")]
        required_float: f64,
    }

    #[test]
    fn test_mixed_fields() {
        let data = json!({
            "required_int": "100",
            "required_float": "2.718"
        });
        let result: TestStruct = serde_json::from_value(data).unwrap();
        assert_eq!(result.required_int, 100);
        assert_eq!(result.required_float, 2.718);
    }

    #[test]
    fn test_required_field_error() {
        let test_cases = vec![
            json!({
                "required_int": "null",
                "required_float": "2.718"
            }),
            json!({
                "required_int": "NULL",
                "required_float": "2.718"
            }),
            json!({
                "required_int": "none",
                "required_float": "2.718"
            }),
            json!({
                "required_int": "NONE",
                "required_float": "2.718"
            }),
            json!({
                "required_int": "",
                "required_float": "2.718"
            }),
            json!({
                "required_int": null,
                "required_float": "2.718"
            }),
        ];

        for (i, test_case) in test_cases.into_iter().enumerate() {
            let result = serde_json::from_value::<TestStruct>(test_case);
            assert!(result.is_err(), "Test case {} should have failed", i);
        }
    }

    #[test]
    fn test_invalid_required_number() {
        let data = json!({
            "required_int": "not_a_number",
            "required_float": "2.718"
        });
        let result = serde_json::from_value::<TestStruct>(data);
        assert!(result.is_err());
    }
}
