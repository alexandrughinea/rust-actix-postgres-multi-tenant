use crate::macros::paginated_query_as::internal::get_postgres_type_casting;

pub trait QueryDialect {
    fn quote_identifier(&self, ident: &str) -> String;
    fn placeholder(&self, position: usize) -> String;
    fn type_cast(&self, value: &str) -> String;
}

pub struct PostgresDialect;

impl QueryDialect for PostgresDialect {
    fn quote_identifier(&self, ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn placeholder(&self, position: usize) -> String {
        format!("${}", position)
    }

    fn type_cast(&self, value: &str) -> String {
        get_postgres_type_casting(value).to_string()
    }
}

pub struct SqliteDialect;

impl QueryDialect for SqliteDialect {
    fn quote_identifier(&self, ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn placeholder(&self, _position: usize) -> String {
        "?".to_string()
    }

    fn type_cast(&self, _value: &str) -> String {
        String::new()
    }
}
