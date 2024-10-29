#[macro_export]
macro_rules! paginated_query_as {
    ($query:expr) => {{
        PaginatedQuery::new(sqlx::query_as($query))
    }};
    ($type:ty, $query:expr) => {{
        PaginatedQuery::new(sqlx::query_as::<_, $type>($query))
    }};
}
