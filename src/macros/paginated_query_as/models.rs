use crate::macros::paginated_query_as::internal::{
    get_struct_field_names, DateRangeParams, PaginationParams, SearchParams, SortDirection,
    SortParams, DEFAULT_DATE_RANGE_COLUMN_NAME, DEFAULT_MAX_PAGE_SIZE, DEFAULT_MIN_PAGE_SIZE,
    DEFAULT_PAGE,
};

use chrono::{DateTime, Utc};
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

impl<T> From<FlatQueryParams> for QueryParams<T> {
    fn from(params: FlatQueryParams) -> Self {
        QueryParams {
            pagination: params.pagination.unwrap_or_default(),
            sort: params.sort.unwrap_or_default(),
            search: params.search.unwrap_or_default(),
            date_range: params.date_range.unwrap_or_default(),
            filters: params.filters.unwrap_or_default(),
            _phantom: PhantomData::<T>,
        }
    }
}

impl<T: Default + Serialize> QueryParams<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> QueryParams<T> {
        self
    }

    pub fn pagination(mut self, page: i64, page_size: i64) -> Self {
        self.pagination = PaginationParams {
            page: page.max(DEFAULT_PAGE),
            page_size: page_size.clamp(DEFAULT_MIN_PAGE_SIZE, DEFAULT_MAX_PAGE_SIZE),
        };
        self
    }

    pub fn sort(mut self, sort_field: impl Into<String>, sort_direction: SortDirection) -> Self {
        self.sort = SortParams {
            sort_column: sort_field.into(),
            sort_direction,
        };
        self
    }

    pub fn search(
        mut self,
        search: impl Into<String>,
        search_columns: Vec<impl Into<String>>,
    ) -> Self {
        self.search = SearchParams {
            search: Some(search.into()),
            search_columns: Some(search_columns.into_iter().map(Into::into).collect()),
        };
        self
    }

    pub fn date_range(
        mut self,
        after: Option<DateTime<Utc>>,
        before: Option<DateTime<Utc>>,
    ) -> Self {
        self.date_range = DateRangeParams {
            date_column: Some(DEFAULT_DATE_RANGE_COLUMN_NAME.to_string()),
            date_after: after,
            date_before: before,
        };
        self
    }

    pub fn filter(mut self, key: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        let key = key.into();
        let valid_fields = get_struct_field_names::<T>();

        if valid_fields.contains(&key) {
            self.filters.insert(key, value.map(Into::into));
        } else {
            tracing::warn!(column = %key, "Skipping invalid filter column");
        }
        self
    }

    pub fn filters(mut self, filters: HashMap<String, Option<impl Into<String>>>) -> Self {
        let valid_fields = get_struct_field_names::<T>();

        self.filters
            .extend(filters.into_iter().filter_map(|(k, v)| {
                if valid_fields.contains(&k) {
                    Some((k, v.map(Into::into)))
                } else {
                    tracing::warn!(column = %k, "Skipping invalid filter column");
                    None
                }
            }));

        self
    }
}
