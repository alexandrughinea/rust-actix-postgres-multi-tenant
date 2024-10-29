use crate::macros::paginated_query_as::QueryParams;
use crate::macros::{get_pg_type_cast, get_struct_field_names, quote_identifier};
use serde::Serialize;
use sqlx::{postgres::PgArguments, Arguments};

pub struct QueryBuilder {
    conditions: Vec<String>,
    arguments: PgArguments,
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            arguments: PgArguments::default(),
        }
    }

    pub fn add_search<T: Default + Serialize>(
        mut self,
        params: &QueryParams<T>,
        valid_columns: &[String],
    ) -> Self {
        if let Some(search) = &params.search.search {
            if let Some(columns) = &params.search.search_columns {
                let valid_search_columns: Vec<&String> = columns
                    .iter()
                    .filter(|column| valid_columns.contains(*column))
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
                        self.arguments.add(pattern);
                    }
                }
            }
        }
        self
    }

    pub fn add_filters<T: Default + Serialize>(
        mut self,
        params: &QueryParams<T>,
        valid_columns: &[String],
    ) -> Self {
        for (key, value) in &params.filters {
            if valid_columns.contains(key) {
                if let Some(value) = value {
                    let table_column = quote_identifier(key);
                    let type_cast = get_pg_type_cast(value);
                    let next_argument = self.arguments.len() + 1;

                    self.conditions.push(format!(
                        "{} = ${}{}",
                        table_column, next_argument, type_cast
                    ));
                    self.arguments.add(value);
                }
            }
        }
        self
    }

    pub fn add_date_range<T>(mut self, params: &QueryParams<T>) -> Self {
        if let Some(after) = params.date_range.created_after {
            let next_argument = self.arguments.len() + 1;
            self.conditions
                .push(format!("created_at >= ${}", next_argument));
            self.arguments.add(after);
        }

        if let Some(before) = params.date_range.created_before {
            let next_argument = self.arguments.len() + 1;
            self.conditions
                .push(format!("created_at <= ${}", next_argument));
            self.arguments.add(before);
        }
        self
    }

    pub fn build(self) -> (Vec<String>, PgArguments) {
        (self.conditions, self.arguments)
    }
}

pub fn build_pg_arguments<T>(params: &QueryParams<T>) -> (Vec<String>, PgArguments)
where
    T: Default + Serialize,
{
    let valid_columns = get_struct_field_names::<T>();

    QueryBuilder::new()
        .add_search(params, &valid_columns)
        .add_filters(params, &valid_columns)
        .add_date_range(params)
        .build()
}
