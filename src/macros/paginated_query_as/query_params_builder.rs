use crate::macros::paginated_query_as::internal::{
    get_struct_field_names, DateRangeParams, PaginationParams, SearchParams, SortDirection,
    SortParams, DEFAULT_DATE_RANGE_COLUMN_NAME, DEFAULT_MAX_PAGE_SIZE, DEFAULT_MIN_PAGE_SIZE,
    DEFAULT_PAGE,
};
use crate::macros::{FlatQueryParams, QueryParams};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::marker::PhantomData;

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

    pub fn add_pagination(mut self, page: i64, page_size: i64) -> Self {
        self.pagination = PaginationParams {
            page: page.max(DEFAULT_PAGE),
            page_size: page_size.clamp(DEFAULT_MIN_PAGE_SIZE, DEFAULT_MAX_PAGE_SIZE),
        };
        self
    }

    pub fn add_sort(
        mut self,
        sort_column: impl Into<String>,
        sort_direction: SortDirection,
    ) -> Self {
        self.sort = SortParams {
            sort_column: sort_column.into(),
            sort_direction,
        };
        self
    }

    pub fn add_search(
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

    pub fn add_date_range(
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

    pub fn add_filter(
        mut self,
        column: impl Into<String>,
        value: Option<impl Into<String>>,
    ) -> Self {
        let table_column = column.into();
        let valid_fields = get_struct_field_names::<T>();

        if valid_fields.contains(&table_column) {
            self.filters.insert(table_column, value.map(Into::into));
        } else {
            tracing::warn!(column = %table_column, "Skipping invalid filter column");
        }
        self
    }

    pub fn add_filters(mut self, filters: HashMap<String, Option<impl Into<String>>>) -> Self {
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
