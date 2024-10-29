use crate::macros::paginated_query_as::internal::{
    get_struct_field_names, quote_identifier, ColumnProtection, PostgresDialect, QueryDialect,
    SqliteDialect,
};
use crate::macros::QueryParams;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Arguments, Database, Encode, Type};
use std::marker::PhantomData;

pub struct QueryBuilder<'q, T, DB: Database> {
    pub conditions: Vec<String>,
    pub arguments: DB::Arguments<'q>,
    valid_columns: Vec<String>,
    protection: Option<ColumnProtection>,
    protection_enabled: bool,
    dialect: Box<dyn QueryDialect>,
    _phantom: PhantomData<&'q T>,
}

impl<'q, T> QueryBuilder<'q, T, sqlx::Postgres>
where
    T: Default + Serialize,
{
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            arguments: sqlx::postgres::PgArguments::default(),
            valid_columns: get_struct_field_names::<T>(),
            protection: Some(ColumnProtection::default()),
            protection_enabled: true,
            dialect: Box::new(PostgresDialect),
            _phantom: PhantomData,
        }
    }
}

impl<'q, T> Default for QueryBuilder<'q, T, sqlx::Sqlite>
where
    T: Default + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'q, T> QueryBuilder<'q, T, sqlx::Sqlite>
where
    T: Default + Serialize,
{
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            arguments: sqlx::sqlite::SqliteArguments::default(),
            valid_columns: get_struct_field_names::<T>(),
            protection: Some(ColumnProtection::default()),
            protection_enabled: true,
            dialect: Box::new(SqliteDialect),
            _phantom: PhantomData,
        }
    }
}

impl<'q, T, DB> QueryBuilder<'q, T, DB>
where
    T: Default + Serialize,
    DB: Database,
    String: for<'a> Encode<'a, DB> + Type<DB>,
{
    pub fn has_column(&self, column: &str) -> bool {
        self.valid_columns.contains(&column.to_string())
    }

    pub fn is_column_safe(&self, column: &str) -> bool {
        let column_exists = self.has_column(column);

        if !self.protection_enabled {
            return column_exists;
        }

        match &self.protection {
            Some(protection) => column_exists && protection.is_safe(column),
            None => column_exists,
        }
    }

    pub fn add_search(mut self, params: &QueryParams<T>) -> Self {
        if let Some(search) = &params.search.search {
            if let Some(columns) = &params.search.search_columns {
                let valid_search_columns: Vec<&String> = columns
                    .iter()
                    .filter(|column| self.is_column_safe(column))
                    .collect();

                if !valid_search_columns.is_empty() && !search.trim().is_empty() {
                    let pattern = format!("%{}%", search);
                    let next_argument = self.arguments.len() + 1;

                    let search_conditions: Vec<String> = valid_search_columns
                        .iter()
                        .map(|column| {
                            let table_column = self.dialect.quote_identifier(column);
                            let placeholder = self.dialect.placeholder(next_argument);
                            format!("LOWER({}) LIKE LOWER({})", table_column, placeholder)
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

    pub fn add_filters(mut self, params: &'q QueryParams<T>) -> Self {
        for (key, value) in &params.filters {
            if self.is_column_safe(key) {
                if let Some(value) = value {
                    let table_column = self.dialect.quote_identifier(key);
                    let type_cast = self.dialect.type_cast(value);
                    let next_argument = self.arguments.len() + 1;
                    let placeholder = self.dialect.placeholder(next_argument);

                    self.conditions
                        .push(format!("{} = {}{}", table_column, placeholder, type_cast));
                    self.arguments.add(value).unwrap_or_default();
                }
            } else {
                tracing::warn!(column = %key, "Skipping invalid filter column");
            }
        }
        self
    }

    pub fn add_date_range(mut self, params: &'q QueryParams<T>) -> Self
    where
        DateTime<Utc>: for<'a> Encode<'a, DB> + Type<DB>,
    {
        if let Some(date_column) = &params.date_range.date_column {
            if self.is_column_safe(date_column) {
                if let Some(after) = params.date_range.date_after {
                    let next_argument = self.arguments.len() + 1;
                    self.conditions
                        .push(format!("{} >= ${}", date_column, next_argument));
                    self.arguments.add(after).unwrap_or_default();
                }

                if let Some(before) = params.date_range.date_before {
                    let next_argument = self.arguments.len() + 1;
                    self.conditions
                        .push(format!("{} <= ${}", date_column, next_argument));
                    self.arguments.add(before).unwrap_or_default();
                }
            } else {
                tracing::warn!(column = %date_column, "Skipping invalid date column");
            }
        }

        self
    }

    pub fn add_condition(
        mut self,
        column: &str,
        condition: impl Into<String>,
        value: String,
    ) -> Self {
        if self.is_column_safe(column) {
            let next_argument = self.arguments.len() + 1;
            self.conditions.push(format!(
                "{} {} ${}",
                quote_identifier(column),
                condition.into(),
                next_argument
            ));
            let _ = self.arguments.add(value);
        } else {
            tracing::warn!(column = %column, "Skipping invalid condition column");
        }
        self
    }

    pub fn add_raw_condition(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    pub fn add_combined_conditions<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut QueryBuilder<T, DB>),
    {
        f(&mut self);
        self
    }

    pub fn disable_protection(mut self) -> Self {
        self.protection_enabled = false;
        self
    }

    pub fn build(self) -> (Vec<String>, DB::Arguments<'q>) {
        (self.conditions, self.arguments)
    }
}
