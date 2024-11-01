mod paginated_query_as;

pub use crate::macros::paginated_query_as::{
    FlatQueryParams, PaginatedQuery, PaginatedResponse, PostgresAdapter, QueryBuilder, QueryParams,
};

pub mod prelude {
    pub use super::{
        FlatQueryParams, PaginatedQuery, PaginatedResponse, PostgresAdapter, QueryBuilder,
        QueryParams,
    };
}
