use crate::macros::paginated_query_as::QueryParams;
use crate::macros::{get_pg_type_cast, get_struct_field_names, quote_identifier};
use serde::Serialize;
use sqlx::{postgres::PgArguments, Arguments, Encode, Postgres};
use std::marker::PhantomData;

/**
* @description
* Only `PgArguments` (PostgresQL) is supported in the first release
*/
pub struct QueryBuilder<T> {
    conditions: Vec<String>,
    arguments: PgArguments, //Support for PgArguments for start.
    valid_columns: Vec<String>,
    _phantom: PhantomData<T>,
}

impl<T> Default for QueryBuilder<T>
where
    T: Default + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> QueryBuilder<T>
where
    T: Default + Serialize,
{
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            arguments: PgArguments::default(),
            valid_columns: get_struct_field_names::<T>(),
            _phantom: PhantomData::<T>,
        }
    }

    pub fn add_search(mut self, params: &QueryParams<T>) -> Self {
        if let Some(search) = &params.search.search {
            if let Some(columns) = &params.search.search_columns {
                let valid_search_columns: Vec<&String> = columns
                    .iter()
                    .filter(|column| self.valid_columns.contains(*column))
                    .collect();

                if !valid_search_columns.is_empty() && !search.trim().is_empty() {
                    let pattern = format!("%{}%", search);
                    let next_argument = self.arguments.len() + 1;

                    let search_conditions: Vec<String> = valid_search_columns
                        .iter()
                        .map(|column| {
                            let table_column = quote_identifier(column);
                            format!("LOWER({}) LIKE LOWER(${next_argument})", table_column)
                        })
                        .collect();

                    if !search_conditions.is_empty() {
                        self.conditions
                            .push(format!("({})", search_conditions.join(" OR ")));
                        self.arguments.add(pattern).unwrap_or_default();
                    }
                }
            }
        }
        self
    }

    pub fn add_filters(mut self, params: &QueryParams<T>) -> Self {
        for (key, value) in &params.filters {
            if self.valid_columns.contains(key) {
                if let Some(value) = value {
                    let table_column = quote_identifier(key);
                    let type_cast = get_pg_type_cast(value);
                    let next_argument = self.arguments.len() + 1;

                    self.conditions.push(format!(
                        "{} = ${}{}",
                        table_column, next_argument, type_cast
                    ));
                    self.arguments.add(value).unwrap_or_default();
                }
            }
        }
        self
    }

    pub fn add_date_range(mut self, params: &QueryParams<T>) -> Self {
        if let Some(after) = params.date_range.created_after {
            let next_argument = self.arguments.len() + 1;
            self.conditions
                .push(format!("created_at >= ${}", next_argument));
            self.arguments.add(after).unwrap_or_default();
        }

        if let Some(before) = params.date_range.created_before {
            let next_argument = self.arguments.len() + 1;
            self.conditions
                .push(format!("created_at <= ${}", next_argument));
            self.arguments.add(before).unwrap_or_default();
        }
        self
    }

    pub fn add_condition<'q, V>(
        mut self,
        column: &str,
        condition: impl Into<String>,
        value: V,
    ) -> Self
    where
        V: Send + Encode<'q, Postgres> + 'static + sqlx::Type<Postgres>,
    {
        if self.valid_columns.contains(&column.to_string()) {
            let next_argument = self.arguments.len() + 1;
            self.conditions.push(format!(
                "{} {} ${}",
                quote_identifier(column),
                condition.into(),
                next_argument
            ));
            let _ = self.arguments.add(value);
        }
        self
    }

    pub fn add_raw_condition(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    pub fn add_combined_conditions<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut QueryBuilder<T>),
    {
        f(&mut self);
        self
    }

    pub fn has_column(&self, column: &str) -> bool {
        self.valid_columns.contains(&column.to_string())
    }

    pub fn build(self) -> (Vec<String>, PgArguments) {
        (self.conditions, self.arguments)
    }
}

pub fn build_query<T>(params: &QueryParams<T>) -> (Vec<String>, PgArguments)
where
    T: Default + Serialize,
{
    QueryBuilder::<T>::new()
        .add_search(params)
        .add_filters(params)
        .add_date_range(params)
        /*
         * description
         * Add extra conditions example if needed (based on the fields/columns on the struct: T)
         */
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
