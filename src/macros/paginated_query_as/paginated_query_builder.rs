/*use crate::macros::paginated_query_as::internal::{quote_identifier, SortDirection};
use crate::macros::{FlatQueryParams, PaginatedResponse, QueryBuilder, QueryParams};
use serde::Serialize;
use sqlx::postgres::PgArguments;
use sqlx::{postgres::Postgres, query::QueryAs, Execute, FromRow, IntoArguments, Pool};

pub struct PaginatedQuery<'q, T, A>
where
    T: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
{
    query: QueryAs<'q, Postgres, T, A>,
    params: QueryParams<T>,
    // Add extra extensibility with a full-blown raw and safe query builder
    build_query_fn: fn(&QueryParams<T>) -> (Vec<String>, PgArguments),
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
            build_query_fn: |params| {
                QueryBuilder::<T, Postgres>::new()
                    .add_search(params)
                    .add_filters(params)
                    .add_date_range(params)
                    .build()
            },
        }
    }

    pub fn with_params(self, params: impl Into<QueryParams<T>>) -> Self {
        Self {
            params: params.into(),
            ..self
        }
    }

    pub fn with_query_builder(
        self,
        build_query_fn: fn(&QueryParams<T>) -> (Vec<String>, PgArguments),
    ) -> Self {
        Self {
            build_query_fn,
            ..self
        }
    }

    pub async fn fetch_paginated(
        self,
        pool: &Pool<Postgres>,
    ) -> Result<PaginatedResponse<T>, sqlx::Error> {
        let base_sql = format!("WITH base_query AS ({})", self.query.sql());
        let (conditions, count_arguments) = (self.build_query_fn)(&self.params);
        let (_, main_arguments) = (self.build_query_fn)(&self.params);

        let where_clause = if !conditions.is_empty() {
            format!(" WHERE {}", conditions.join(" AND "))
        } else {
            String::new()
        };

        let mut main_sql = format!("{} SELECT * FROM base_query{}", base_sql, where_clause);
        let pagination = self.params.pagination.clone();
        let sort = self.params.sort.clone();

        let page_size = pagination.page_size;
        let page = pagination.page;
        let order = match sort.sort_direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };

        main_sql.push_str(&format!(
            " ORDER BY {} {}",
            quote_identifier(&sort.sort_column),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::paginated_query_as::internal::SortDirection;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;

    #[derive(Debug, Default, Serialize)]
    struct TestModel {
        name: String,
        title: String,
        description: String,
        status: String,
        category: String,
        updated_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    }

    #[test]
    fn test_empty_params() {
        let params = QueryParams::<TestModel>::new().build();

        assert_eq!(params.pagination.page, 1);
        assert_eq!(params.pagination.page_size, 10);
        assert_eq!(params.sort.sort_column, "created_at");
        assert!(matches!(
            params.sort.sort_direction,
            SortDirection::Descending
        ));
    }

    #[test]
    fn test_partial_params() {
        let params = QueryParams::<TestModel>::new()
            .pagination(2, 10)
            .search("test".to_string(), vec!["name".to_string()])
            .build();

        assert_eq!(params.pagination.page, 2);
        assert_eq!(params.search.search, Some("test".to_string()));
        assert_eq!(params.pagination.page_size, 10);
        assert_eq!(params.sort.sort_column, "created_at");
        assert!(matches!(
            params.sort.sort_direction,
            SortDirection::Descending
        ));
    }

    #[test]
    fn test_invalid_params() {
        // For builder pattern, invalid params would be handled at compile time
        // But we can test the defaults
        let params = QueryParams::<TestModel>::new()
            .pagination(0, 0) // Should be clamped to minimum values
            .build();

        assert_eq!(params.pagination.page, 1);
        assert_eq!(params.pagination.page_size, 10);
    }

    #[test]
    fn test_filters() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), Some("active".to_string()));
        filters.insert("category".to_string(), Some("test".to_string()));

        let params = QueryParams::<TestModel>::new().filters(filters).build();

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
        let params = QueryParams::<TestModel>::new()
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
        let params = QueryParams::<TestModel>::new()
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
        assert_eq!(params.sort.sort_column, "updated_at");
        assert!(matches!(
            params.sort.sort_direction,
            SortDirection::Ascending
        ));
        assert_eq!(params.search.search, Some("test".to_string()));
        assert_eq!(
            params.search.search_columns,
            Some(vec!["title".to_string(), "description".to_string()])
        );
        assert!(params.date_range.date_after.is_some());
        assert!(params.date_range.date_before.is_none());
    }

    #[test]
    fn test_filter_chain() {
        let params = QueryParams::<TestModel>::new()
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
        let params = QueryParams::<TestModel>::new()
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
*/
