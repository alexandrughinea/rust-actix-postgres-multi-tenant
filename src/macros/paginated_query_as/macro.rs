#[macro_export]
macro_rules! paginated_query_as {
    ($query:expr) => {{
        let params = QueryParams::<_>::new().build();
        let query_params_result = QueryBuilder::<_, Postgres>::new()
            .add_search(&params)
            .add_filters(&params)
            .add_date_range(&params)
            .build();
        let adapter = PostgresAdapter::new(query_params_result);
        let sqlx_query_as = sqlx::query_as::<Postgres, _>($query).sql().to_string();

        PaginatedQuery::<_, Postgres, _>::new(adapter, sqlx_query_as)
    }};
    ($type:ty, $query:expr) => {{
        let params = QueryParams::<$type>::new().build();
        let query_params_result = QueryBuilder::<$type, Postgres>::new()
            .add_search(&params)
            .add_filters(&params)
            .add_date_range(&params)
            .build();
        let adapter = PostgresAdapter::new(query_params_result);
        let sqlx_query_as = sqlx::query_as::<Postgres, $type>($query).sql().to_string();

        PaginatedQuery::<$type, Postgres, _>::new(adapter, sqlx_query_as)
    }};
}
