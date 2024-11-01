use crate::macros::paginated_query_as::internal::{
    DateRangeParams, PaginationParams, SearchParams, SortParams,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedResponse<T> {
    pub records: Vec<T>,
    pub total: i64,
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
pub struct QueryParams<T> {
    pub pagination: PaginationParams,
    pub sort: SortParams,
    pub search: SearchParams,
    pub date_range: DateRangeParams,
    pub filters: HashMap<String, Option<String>>,
    pub(crate) _phantom: PhantomData<T>,
}
