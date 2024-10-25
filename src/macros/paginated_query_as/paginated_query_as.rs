use crate::macros::paginated_query_as::{
    build_pg_arguments, quote_identifier, DateRangeParams, FlatQueryParams, PaginatedResponse,
    PaginationParams, QueryParams, SearchParams, SortDirection, SortParams,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{postgres::Postgres, query::QueryAs, Execute, FromRow, IntoArguments, Pool, Row};
use std::collections::HashMap;

impl From<FlatQueryParams> for QueryParams {
    fn from(params: FlatQueryParams) -> Self {
        QueryParams {
            pagination: params.pagination.unwrap_or_default(),
            sort: params.sort.unwrap_or_default(),
            search: params.search.unwrap_or_default(),
            date_range: params.date_range.unwrap_or_default(),
            filters: params.filters.unwrap_or_default(),
        }
    }
}

impl QueryParams {
    pub fn pagination(mut self, page: i64, page_size: i64) -> Self {
        self.pagination = PaginationParams {
            page: page.max(1),
            page_size: page_size.clamp(1, 100),
        };
        self
    }

    pub fn sort(mut self, sort_field: impl Into<String>, sort_direction: SortDirection) -> Self {
        self.sort = SortParams {
            sort_field: sort_field.into(),
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
            created_after: after,
            created_before: before,
        };
        self
    }

    pub fn filter(mut self, key: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        self.filters.insert(key.into(), value.map(Into::into));
        self
    }

    pub fn filters(mut self, filters: HashMap<String, Option<impl Into<String>>>) -> Self {
        self.filters
            .extend(filters.into_iter().map(|(k, v)| (k, v.map(Into::into))));

        self
    }

    pub fn build(self) -> QueryParams {
        QueryParams {
            pagination: self.pagination,
            sort: self.sort,
            search: self.search,
            date_range: self.date_range,
            filters: self.filters,
        }
    }
}

pub struct PaginatedQuery<'q, T, A>
where
    T: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
{
    query: QueryAs<'q, Postgres, T, A>,
    params: QueryParams,
}

impl<'q, T, A> PaginatedQuery<'q, T, A>
where
    T: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row>
        + Send
        + Unpin
        + Serialize
        + Default
        + 'static,
    A: 'q + IntoArguments<'q, Postgres> + Send,
{
    pub fn new(query: QueryAs<'q, Postgres, T, A>) -> Self {
        Self {
            query,
            params: FlatQueryParams::default().into(),
        }
    }

    pub fn with_params(self, params: impl Into<QueryParams>) -> Self {
        Self {
            query: self.query,
            params: params.into(),
        }
    }

    pub async fn fetch_paginated(
        self,
        pool: &Pool<Postgres>,
    ) -> Result<PaginatedResponse<T>, sqlx::Error> {
        let base_sql = format!("WITH base_query AS ({})", self.query.sql());

        let (conditions, count_arguments) = build_pg_arguments(&self.params);
        let (_, main_arguments) = build_pg_arguments(&self.params);

        let where_clause = if !conditions.is_empty() {
            format!(" WHERE {}", conditions.join(" AND "))
        } else {
            String::new()
        };

        let mut main_sql = format!("{} SELECT * FROM base_query{}", base_sql, where_clause);
        let pagination = self.params.pagination.clone();
        let sort = self.params.sort.clone();

        let page_size = pagination.page_size.clamp(1, 100);
        let page = pagination.page.max(1);
        let order = match sort.sort_direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };

        main_sql.push_str(&format!(
            " ORDER BY {} {}",
            quote_identifier(&sort.sort_field),
            order
        ));

        main_sql.push_str(&format!(
            " LIMIT {} OFFSET {}",
            page_size,
            (page - 1) * page_size
        ));

        let count_sql = format!(
            "{} SELECT COUNT(*) FROM base_query{}",
            base_sql, where_clause
        );

        let total: i64 = sqlx::query_scalar_with(&count_sql, count_arguments)
            .fetch_one(pool)
            .await?;
        let total_pages = if total == 0 {
            0
        } else {
            (total + page_size - 1) / page_size
        };

        let records = sqlx::query_as_with::<_, T, _>(&main_sql, main_arguments)
            .fetch_all(pool)
            .await?;

        Ok(PaginatedResponse {
            records,
            total,
            pagination,
            total_pages,
        })
    }
}

#[macro_export]
macro_rules! paginated_query_as {
    ($query:expr) => {{
        PaginatedQuery::new(sqlx::query_as($query))
    }};
    ($type:ty, $query:expr) => {{
        PaginatedQuery::new(sqlx::query_as::<_, $type>($query))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_params() {
        let params = QueryParams::default().build();

        assert_eq!(params.pagination.page, 1);
        assert_eq!(params.pagination.page_size, 10);
        assert_eq!(params.sort.sort_field, "created_at");
        assert!(matches!(
            params.sort.sort_direction,
            SortDirection::Descending
        ));
    }

    #[test]
    fn test_partial_params() {
        let params = QueryParams::default()
            .pagination(2, 10)
            .search("test".to_string(), vec!["name".to_string()])
            .build();

        assert_eq!(params.pagination.page, 2);
        assert_eq!(params.search.search, Some("test".to_string()));
        assert_eq!(params.pagination.page_size, 10);
        assert_eq!(params.sort.sort_field, "created_at");
        assert!(matches!(
            params.sort.sort_direction,
            SortDirection::Descending
        ));
    }

    #[test]
    fn test_invalid_params() {
        // For builder pattern, invalid params would be handled at compile time
        // But we can test the defaults
        let params = QueryParams::default()
            .pagination(0, 0) // Should be clamped to minimum values
            .build();

        assert_eq!(params.pagination.page, 1);
        assert_eq!(params.pagination.page_size, 1);
    }

    #[test]
    fn test_filters() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), Some("active".to_string()));
        filters.insert("category".to_string(), Some("test".to_string()));

        let params = QueryParams::default().filters(filters).build();

        assert!(params.filters.contains_key("status"));
        assert_eq!(
            params.filters.get("status").unwrap(),
            &Some("active".to_string())
        );
        assert!(params.filters.contains_key("category"));
        assert_eq!(
            params.filters.get("category").unwrap(),
            &Some("test".to_string())
        );
    }

    #[test]
    fn test_search_with_columns() {
        let params = QueryParams::default()
            .search(
                "test".to_string(),
                vec!["title".to_string(), "description".to_string()],
            )
            .build();

        assert_eq!(params.search.search, Some("test".to_string()));
        assert_eq!(
            params.search.search_columns,
            Some(vec!["title".to_string(), "description".to_string()])
        );
    }

    #[test]
    fn test_full_params() {
        let params = QueryParams::default()
            .pagination(2, 20)
            .sort("updated_at".to_string(), SortDirection::Ascending)
            .search(
                "test".to_string(),
                vec!["title".to_string(), "description".to_string()],
            )
            .date_range(Some(Utc::now()), None)
            .build();

        assert_eq!(params.pagination.page, 2);
        assert_eq!(params.pagination.page_size, 20);
        assert_eq!(params.sort.sort_field, "updated_at");
        assert!(matches!(
            params.sort.sort_direction,
            SortDirection::Ascending
        ));
        assert_eq!(params.search.search, Some("test".to_string()));
        assert_eq!(
            params.search.search_columns,
            Some(vec!["title".to_string(), "description".to_string()])
        );
        assert!(params.date_range.created_after.is_some());
        assert!(params.date_range.created_before.is_none());
    }

    #[test]
    fn test_search_query_generation() {
        let params = QueryParams::default()
            .search("XXX".to_string(), vec!["name".to_string()])
            .build();

        let (conditions, _) = build_pg_arguments(&params);

        assert!(!conditions.is_empty());
        assert!(conditions.iter().any(|c| c.contains("LOWER")));
        assert!(conditions.iter().any(|c| c.contains("LIKE LOWER")));
    }

    #[test]
    fn test_empty_search_query() {
        let params = QueryParams::default()
            .search("   ".to_string(), vec!["name".to_string()])
            .build();

        let (conditions, _) = build_pg_arguments(&params);
        assert!(!conditions.iter().any(|c| c.contains("LIKE")));
    }

    #[test]
    fn test_filter_chain() {
        let params = QueryParams::default()
            .filter("status", Some("active"))
            .filter("category", Some("test"))
            .build();

        assert_eq!(
            params.filters.get("status").unwrap(),
            &Some("active".to_string())
        );
        assert_eq!(
            params.filters.get("category").unwrap(),
            &Some("test".to_string())
        );
    }

    #[test]
    fn test_mixed_pagination() {
        let params = QueryParams::default()
            .pagination(2, 10)
            .search("test".to_string(), vec!["title".to_string()])
            .filter("status", Some("active"))
            .build();

        assert_eq!(params.pagination.page, 2);
        assert_eq!(params.pagination.page_size, 10);
        assert_eq!(params.search.search, Some("test".to_string()));
        assert_eq!(
            params.filters.get("status").unwrap(),
            &Some("active".to_string())
        );
    }
}