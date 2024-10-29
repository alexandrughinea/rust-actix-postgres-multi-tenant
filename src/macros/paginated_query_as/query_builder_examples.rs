use crate::macros::{QueryBuilder, QueryParams};
use serde::Serialize;
use sqlx::postgres::PgArguments;
use sqlx::sqlite::SqliteArguments;
use sqlx::Arguments;
use sqlx::{Postgres, Sqlite};

#[allow(dead_code)]
pub fn builder_new_query_with_disabled_protection_for_sqlite<T>(
    params: &QueryParams<T>,
) -> (Vec<String>, SqliteArguments)
where
    T: Default + Serialize,
{
    QueryBuilder::<T, Sqlite>::new()
        .add_search(params)
        .add_filters(params)
        .add_date_range(params)
        .disable_protection()
        .build()
}

#[allow(dead_code)]
pub fn build_query_with_disabled_protection<T>(
    params: &QueryParams<T>,
) -> (Vec<String>, PgArguments)
where
    T: Default + Serialize,
{
    QueryBuilder::<T, Postgres>::new()
        .add_search(params)
        .add_filters(params)
        .add_date_range(params)
        .disable_protection()
        .add_combined_conditions(|builder| {
            if builder.has_column("status") && builder.has_column("role") {
                builder
                    .conditions
                    .push("(status = 'active' AND role IN ('admin', 'user'))".to_string());
            }
            if builder.has_column("score") {
                builder
                    .conditions
                    .push("score BETWEEN $1 AND $2".to_string());
                let _ = builder.arguments.add(50);
                let _ = builder.arguments.add(100);
            }
        })
        .build()
}

#[allow(dead_code)]
pub fn build_query_with_safe_defaults<T>(params: &QueryParams<T>) -> (Vec<String>, PgArguments)
where
    T: Default + Serialize,
{
    QueryBuilder::<T, Postgres>::new()
        .add_search(params)
        .add_filters(params)
        .add_date_range(params)
        /*
         * description
         * Add extra conditions example if needed (based on the fields/columns on the struct: T)
        .add_combined_conditions(|builder| {
            if builder.has_column("status") && builder.has_column("role") {
                builder
                    .conditions
                    .push("(status = 'active' AND role IN ('admin', 'user'))".to_string());
            }
            if builder.has_column("score") {
                builder
                    .conditions
                    .push("score BETWEEN $1 AND $2".to_string());
                let _ = builder.arguments.add(50);
                let _ = builder.arguments.add(100);
            }
        })*/
        .build()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::macros::QueryParams;
    use chrono::{DateTime, Utc};

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
    fn test_search_query_generation() {
        let params = QueryParams::new()
            .search("XXX".to_string(), vec!["name".to_string()])
            .build();

        let (conditions, _) = build_query_with_safe_defaults::<TestModel>(&params);

        assert!(!conditions.is_empty());
        assert!(conditions.iter().any(|c| c.contains("LOWER")));
        assert!(conditions.iter().any(|c| c.contains("LIKE LOWER")));
    }

    #[test]
    fn test_empty_search_query() {
        let params = QueryParams::new()
            .search("   ".to_string(), vec!["name".to_string()])
            .build();

        let (conditions, _) = build_query_with_safe_defaults::<TestModel>(&params);
        assert!(!conditions.iter().any(|c| c.contains("LIKE")));
    }
}
