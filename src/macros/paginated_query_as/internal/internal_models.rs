use crate::macros::paginated_query_as::internal::{
    default_date_range_column, default_page, default_page_size, default_search_columns,
    default_sort_column, default_sort_direction, deserialize_page, deserialize_page_size,
    deserialize_search, deserialize_search_columns,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct PaginationParams {
    #[serde(deserialize_with = "deserialize_page", default = "default_page")]
    pub page: i64,
    #[serde(
        deserialize_with = "deserialize_page_size",
        default = "default_page_size"
    )]
    pub page_size: i64,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SortParams {
    #[serde(default = "default_sort_column")]
    pub sort_column: String,
    #[serde(default = "default_sort_direction")]
    pub sort_direction: SortDirection,
}

impl Default for SortParams {
    fn default() -> Self {
        Self {
            sort_column: default_sort_column(),
            sort_direction: default_sort_direction(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SearchParams {
    #[serde(deserialize_with = "deserialize_search")]
    pub search: Option<String>,
    #[serde(
        deserialize_with = "deserialize_search_columns",
        default = "default_search_columns"
    )]
    pub search_columns: Option<Vec<String>>,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            search: None,
            search_columns: default_search_columns(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub struct DateRangeParams {
    #[serde(default = "default_date_range_column")]
    pub date_column: Option<String>,
    pub date_after: Option<DateTime<Utc>>,
    pub date_before: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum SortDirection {
    Ascending,
    #[default]
    Descending,
}
