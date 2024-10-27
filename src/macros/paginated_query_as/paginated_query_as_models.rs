use crate::macros::paginated_query_as::{
    default_page, default_page_size, default_search_columns, default_sort_direction,
    default_sort_field, deserialize_page, deserialize_page_size, deserialize_search,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedResponse<T> {
    pub records: Vec<T>,
    pub total: i64,
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub total_pages: i64,
}

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
    #[serde(default = "default_sort_field")]
    pub sort_field: String,
    #[serde(default = "default_sort_direction")]
    pub sort_direction: SortDirection,
}

impl Default for SortParams {
    fn default() -> Self {
        Self {
            sort_field: default_sort_field(),
            sort_direction: default_sort_direction(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SearchParams {
    #[serde(deserialize_with = "deserialize_search")]
    pub search: Option<String>,
    #[serde(default = "default_search_columns")]
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
    #[serde(rename = "created_after")]
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Ascending,
    #[default]
    Descending,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Validate)]
pub struct FlatQueryParams {
    #[serde(flatten)]
    pub pagination: Option<PaginationParams>,
    #[serde(flatten)]
    pub sort: Option<SortParams>,
    #[serde(flatten)]
    pub search: Option<SearchParams>,
    #[serde(flatten)]
    pub date_range: Option<DateRangeParams>,
    #[serde(flatten)]
    pub filters: Option<HashMap<String, Option<String>>>,
}

#[derive(Debug, Default)]
pub struct QueryParams {
    pub pagination: PaginationParams,
    pub sort: SortParams,
    pub search: SearchParams,
    pub date_range: DateRangeParams,
    pub filters: HashMap<String, Option<String>>,
}

impl QueryParams {
    pub fn new() -> Self {
        Self::default()
    }
}
