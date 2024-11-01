use crate::macros::paginated_query_as::internal::{PaginationParams, SortDirection};
use crate::macros::paginated_query_as::query_builder_examples::{
    build_query_with_disabled_protection, builder_new_query_with_disabled_protection_for_sqlite,
};
use crate::macros::{FlatQueryParams, PaginatedResponse, QueryBuilder, QueryParams};
use crate::models::User;
use serde::Serialize;
use sqlx::postgres::PgArguments;
use sqlx::Arguments;
use sqlx::{Database, Execute, FromRow, IntoArguments, Pool, Postgres};
use std::marker::PhantomData;

pub trait DbAdapter<'q, DB: Database, T>
where
    T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
{
    type Args: IntoArguments<'q, DB> + Send;

    fn build_where_clause(&mut self) -> (String, Self::Args);
    fn build_order_by(&self, params: &QueryParams<T>) -> String;
    fn build_count_query(
        &mut self,
        base_sql: &str,
        params: &QueryParams<T>,
    ) -> (String, Self::Args);
    fn build_main_query(&mut self, base_sql: &str, params: &QueryParams<T>)
        -> (String, Self::Args);

    async fn query_scalar(
        &self,
        query: &'q str,
        args: Self::Args,
        pool: &Pool<DB>,
    ) -> Result<i64, sqlx::Error>;

    async fn query_as(
        &self,
        query: &'q str,
        args: Self::Args,
        pool: &Pool<DB>,
    ) -> Result<Vec<T>, sqlx::Error>;
}

pub struct PostgresAdapter<'q, T> {
    conditions: Vec<String>,
    arguments: PgArguments,
    _phantom: PhantomData<&'q T>,
}

impl<'q, T> PostgresAdapter<'q, T> {
    pub fn new((conditions, arguments): (Vec<String>, PgArguments)) -> Self {
        Self {
            conditions,
            arguments,
            _phantom: PhantomData,
        }
    }
}

impl<'q, T> DbAdapter<'q, Postgres, T> for PostgresAdapter<'q, T>
where
    T: for<'r> FromRow<'r, <Postgres as Database>::Row>
        + Send
        + Unpin
        + Default
        + Serialize
        + 'static,
{
    type Args = PgArguments;

    fn build_where_clause(&mut self) -> (String, Self::Args) {
        // Create a new arguments instance
        let mut new_args = PgArguments::default();
        // Take ownership of the current arguments
        std::mem::swap(&mut self.arguments, &mut new_args);

        if self.conditions.is_empty() {
            (String::new(), new_args)
        } else {
            (
                format!(" WHERE {}", self.conditions.join(" AND ")),
                new_args,
            )
        }
    }

    fn build_order_by(&self, params: &QueryParams<T>) -> String {
        format!(
            " ORDER BY {} {}",
            params.sort.sort_column,
            match params.sort.sort_direction {
                SortDirection::Ascending => "ASC",
                SortDirection::Descending => "DESC",
            }
        )
    }

    fn build_count_query(
        &mut self,
        base_sql: &str,
        _params: &QueryParams<T>,
    ) -> (String, Self::Args) {
        let (where_clause, args) = self.build_where_clause();
        let query = format!(
            "SELECT COUNT(*) FROM ({}) AS base_query{}",
            base_sql, where_clause
        );
        (query, args)
    }

    fn build_main_query(
        &mut self,
        base_sql: &str,
        params: &QueryParams<T>,
    ) -> (String, Self::Args) {
        let (where_clause, args) = self.build_where_clause();
        let order_by = self.build_order_by(params);

        let query = format!(
            "SELECT * FROM ({}) AS base_query{} {} LIMIT {} OFFSET {}",
            base_sql,
            where_clause,
            order_by,
            params.pagination.page_size,
            (params.pagination.page - 1) * params.pagination.page_size
        );

        (query, args)
    }

    async fn query_scalar(
        &self,
        query: &'q str,
        args: Self::Args,
        pool: &Pool<Postgres>,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar_with::<_, i64, _>(query, args)
            .fetch_one(pool)
            .await
    }

    async fn query_as(
        &self,
        query: &'q str,
        args: Self::Args,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<T>, sqlx::Error> {
        sqlx::query_as_with::<Postgres, T, _>(query, args)
            .fetch_all(pool)
            .await
    }
}

pub struct PaginatedQuery<'q, T, DB, Adapter>
where
    DB: Database,
    T: for<'r> FromRow<'r, DB::Row> + Send + Unpin + 'static,
    Adapter: DbAdapter<'q, DB, T>,
{
    pub adapter: Adapter,
    pub base_sql: String,
    pub count_query: String,
    pub main_query: String,
    pub params: QueryParams<T>,
    _marker: PhantomData<(&'q DB, T)>,
}

impl<'q, T, DB, Adapter> PaginatedQuery<'q, T, DB, Adapter>
where
    DB: Database,
    T: for<'r> FromRow<'r, DB::Row> + Send + Unpin + 'static,
    Adapter: DbAdapter<'q, DB, T>,
{
    pub fn new(mut adapter: Adapter, base_sql: String) -> Self {
        let params: QueryParams<T> = FlatQueryParams::default().into();
        let (count_query, count_args) = adapter.build_count_query(&base_sql, &params);
        let (main_query, main_args) = adapter.build_main_query(&base_sql, &params);

        Self {
            adapter,
            base_sql,
            count_query,
            main_query,
            params,
            _marker: PhantomData,
        }
    }

    pub fn with_params(mut self, params: impl Into<QueryParams<T>>) -> Self {
        let params = params.into();
        let (count_query, count_args) = self.adapter.build_count_query(&self.base_sql, &params);
        let (main_query, main_args) = self.adapter.build_main_query(&self.base_sql, &params);

        Self {
            count_query,
            main_query,
            params,
            ..self
        }
    }

    pub fn with_adapter(self, adapter: Adapter) -> Self {
        Self { adapter, ..self }
    }

    pub async fn fetch_paginated(
        &'q mut self,
        pool: &Pool<DB>,
    ) -> Result<PaginatedResponse<T>, sqlx::Error> {
        let (_, count_args) = self.adapter.build_count_query(&self.base_sql, &self.params);
        let (_, rows_args) = self.adapter.build_main_query(&self.base_sql, &self.params);

        let total = self
            .adapter
            .query_scalar(&self.count_query, count_args, pool)
            .await?;

        let records = self
            .adapter
            .query_as(&self.main_query, rows_args, pool)
            .await?;

        Ok(PaginatedResponse {
            records,
            total,
            pagination: self.params.pagination.clone(),
            total_pages: (total + self.params.pagination.page_size - 1)
                / self.params.pagination.page_size,
        })
    }
}

fn test_stuff() {
    let params = QueryParams::<User>::new()
        .add_search("xx".to_string(), vec!["first_name".to_string()])
        .add_filter("confirmed", Some("true"))
        .build();

    let query_postgres_example_result = QueryBuilder::<User, Postgres>::new()
        .add_search(&params)
        .build();
    let query_postgres_custom_example_result = build_query_with_disabled_protection(&params);
    let query_sqlite_custom_example_result =
        builder_new_query_with_disabled_protection_for_sqlite(&params);

    let adapter = PostgresAdapter::new(query_postgres_example_result);
    let sqlx_query_as = sqlx::query_as::<Postgres, User>("SELECT * from users")
        .sql()
        .to_string();

    PaginatedQuery::<User, Postgres, _>::new(adapter, sqlx_query_as);
}
