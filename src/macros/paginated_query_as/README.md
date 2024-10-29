# Paginated Query Builder for SQLx PostgreSQL

A flexible, type-safe thin layer over SQLx query builder for handling paginated queries in Rust and PostgreSQL (can be extended).
Built for robust handling of dynamic web API queries with built-in support for pagination, searching, filtering, and sorting 

## Table of Contents
- [Features](#features)
- [Market analysis](#market-analysis)
- [Why Use This Library](#why-this-library)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Documentation](#api-documentation)
- [Framework Integration](#framework-integration)
- [Advanced Usage](#advanced-usage)
- [Best Practices](#best-practices)
- [Contributing](#contributing)
- [License](#license)

## Features

### Core capabilities
- 🔍 Full-text search with column specification
- 📑 Smart pagination with customizable page size
- 🔄 Dynamic sorting on any column
- 🎯 Flexible filtering system
- 📅 Date range filtering
- 🔒 Type-safe operations
- ⚡ High performance
- 🛡️ SQL injection protection

### Technical features
- Builder pattern for query construction
- Automatic PostgreSQL type casting
- Error handling and logging
- Serialization/deserialization support
- Custom base query support

### Query features
- Case-insensitive search
- Multiple column search
- Complex filtering conditions
- Date-based filtering
- Dynamic sort direction
- Customizable page size
- Result count optimization

## Market analysis

### Current ecosystem gaps

1. **Query builders**
   - Diesel: Full ORM, can be heavyweight
   - SQLx: Great low-level toolkit but no high-level query building
   - SeaQuery: Generic and verbose, SQL builder but lacks PostgreSQL-specific features
   - sqlbuilder: Basic SQL building without pagination or security

2. **Missing features in existing solutions**
   - Type-safe pagination
   - Built-in SQL injection protection
   - System table access control
   - Easy integration with web frameworks
   - Automatic type casting
   - Typesafe search/filter/sort/pagination capabilities

### Unique selling points

1. **Security-first Approach**
   ```rust
   let builder = ProtectedQueryBuilder::new()
       .add_condition("name", "=", "value")  // Safe by default
       .add_condition("ctid", "=", "1.1");   // Protected system columns
   ```

2. **Web framework integration and extensible thin sqlx layer**
   ```rust
   // Actix Web example
   async fn list_users(Query(params): Query<QueryParams>) -> impl Responder {
       let query = paginated_query_as!(User, "SELECT * FROM users")
           .with_params(params)
           .fetch_paginated(&pool);
   }
   ```

3. **Type safety & ergonomics**
   ```rust
   // Type inference and validation
   let params = QueryParams::<User>::new()
       .pagination(1, 10)
       .sort("created_at", SortDirection::Descending)
       .search("john", vec!["name", "email"]);
   ```

### Target audience

1. **Primary users**
   - Rust web developers using PostgreSQL
   - Teams needing secure query building
   - Projects requiring pagination APIs
   - SQLx users wanting higher-level abstractions

2. **Use cases**
   - REST APIs with pagination
   - Admin panels
   - Data exploration interfaces

## Why this library

### Problems solved
1. **Boilerplate Reduction**
   - Eliminates repetitive query building code
   - Standardizes pagination handling
   - Simplifies API parameter processing

2. **Security**
   - Prevents SQL injection attacks
   - Handles input sanitization
   - Manages parameter binding safely

3. **Performance**
   - Optimized query generation
   - Efficient pagination handling
   - Smart count queries

4. **Flexibility**
   - Works with any PostgreSQL schema
   - Supports complex queries
   - Adaptable to various use cases


### Compared to alternatives
- Type-safe compared to raw SQL
- More flexible than ORM solutions
- Better performance than query builders
- Designed for web API use cases

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls", "chrono", "uuid"] }
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Quick start

```rust
use sqlx_addons::paginated_query_as;

#[derive(sqlx::FromRow, serde::Serialize)]
struct User {
    id: i64,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
}

async fn get_users(pool: &PgPool) -> Result<PaginatedResponse<User>, sqlx::Error> {
    paginated_query_as!(User, "SELECT * FROM users")
        .with_params(QueryParams::<User>::new()
            .pagination(1, 10)
            .sort("created_at", SortDirection::Descending)
            .search("john", vec!["name", "email"])
            .build())
        .fetch_paginated(pool)
        .await
}
```

## API documentation

### Defaults & constraints overview
The query builder comes with predefined defaults to ensure consistent behavior and performance across your application. These defaults can be customized based on your needs while maintaining safe boundaries for your API.

### Pagination defaults
| Constant | Value | Description |
|----------|---------|-------------|
| `DEFAULT_PAGE` | 1 | Starting page number |
| `DEFAULT_MIN_PAGE_SIZE` | 10 | Minimum items per page |
| `DEFAULT_MAX_PAGE_SIZE` | 50 | Maximum items per page |

### Field constraints
| Constant | Value | Description |
|----------|---------|-------------|
| `DEFAULT_MIN_FIELD_LENGTH` | 1 | Minimum search term length |
| `DEFAULT_MAX_FIELD_LENGTH` | 100 | Maximum search term length |

### Search configuration
```rust
// Default columns used for search operations when none specified
pub const DEFAULT_SEARCH_COLUMNS: [&str; 2] = [
    "name",
    "description"
];
```

### Default behaviors

#### 🔄 Pagination
- Starts at page 1 if not specified
- Uses minimum page size (10) if not specified
- Automatically clamps page size between 10 and 50
- Returns total count and total pages

#### 🔍 Search
- Ignores empty search terms
- Truncates terms exceeding 100 characters
- Falls back to default columns if none specified
- Case-insensitive by default

#### 📑 Sorting
- Default field: `"created_at"`
- Default direction: `Descending`
- Validates sort fields against model

### Examples

#### Using defaults
```rust
let params = QueryParams::<User>::new().build();
// Results in:
// - Page 1
// - 10 items per page
// - Sorted by created_at DESC
```

#### Customizing defaults
```rust
let params = QueryParams::<User>::new()
    .pagination(2, 30)          // Page 2, 30 items
    .search(                    // Custom search
        "john",
        vec!["email", "username"]
    )
    .sort(                      // Custom sort
        "last_login",
        SortDirection::Ascending
    )
    .build();
```

#### Working with constraints
```rust
// Page size will be clamped
let params = QueryParams::<User>::new()
    .pagination(1, 100)  // Will be clamped to 50
    .build();

// Search term will be truncated
let long_search = "a".repeat(150);  // Will be truncated to 100 chars
let params = QueryParams::<User>::new()
    .search(long_search, vec!["name"])
    .build();
```

### HTTP query Parameters
```
GET /api/users
    ?page=1               // Defaults to 1
    &page_size=20        // Clamped between 10-50
    &sort_field=name     // Optional, defaults to created_at
    &sort_direction=asc  // Optional, defaults to desc
    &search=john         // Optional
    &search_columns=email,username  // Optional
```

### Chaining methods

#### Pagination
```rust
/// Set page number and size
fn pagination(page: i64, page_size: i64) -> Self

// Example
.pagination(1, 10) // Page 1, 10 items per page
```

#### Sorting
```rust
/// Set sort field and direction
fn sort(field: impl Into<String>, direction: SortDirection) -> Self

// Example
.sort("created_at", SortDirection::Descending)
```

#### Search
```rust
/// Set search term and columns
fn search(term: impl Into<String>, columns: Vec<impl Into<String>>) -> Self

// Example
.search("john", vec!["name", "email"])
```

#### Filtering
```rust
/// Add single filter
fn filter(key: impl Into<String>, value: Option<impl Into<String>>) -> Self

/// Add multiple filters
fn filters(filters: HashMap<String, Option<String>>) -> Self

// Example
.filter("status", Some("active"))
```

#### Date range
```rust
/// Set date range for filtering
fn date_range(after: Option<DateTime<Utc>>, before: Option<DateTime<Utc>>) -> Self

// Example
.date_range(Some(Utc::now() - Duration::days(7)), None)
```

### Response type
```rust
pub struct PaginatedResponse<T> {
    pub records: Vec<T>,
    pub total: i64,
    pub pagination: PaginationParams,
    pub total_pages: i64,
}
```

### Query parameters from HTTP
```
GET /api/users?
    page=1&
    page_size=10&
    sort_field=created_at&
    sort_direction=descending&
    search=john&
    search_columns=name,email&
    status=active
```

### Response format
```json
{
  "records": [...],
  "total": 100,
  "page": 1,
  "page_size": 10,
  "total_pages": 10
}
```

## Framework integration

### Actix Web integration

#### Setup
```toml
[dependencies]
actix-web = "4.*"
```

#### Basic Actix handler
```rust
use actix_web::{web, Data, Query, HttpResponse};

pub async fn get_users(
    pool: Data<PgPool>,
    Query(params): Query<FlatQueryParams>,
) -> HttpResponse {
    match paginated_query_as!(User, "SELECT * FROM users")
        .with_params(params)
        .fetch_paginated(&pool)
        .await
    {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch users"
            }))
        }
    }
}
```

#### Advanced handler with extended filters and extra SQL logic
Notice how the params from the `Data` extractor (extracted from the request) can be extended further programmatically with extra filters.
```rust
use actix_web::{web, Data, Query, HttpResponse};

pub async fn get_users_custom(
    req: HttpRequest,
    pool: Data<PgPool>,
    Query(params): Query<FlatQueryParams>,
) -> Result<HttpResponse, Error> {
    let params = QueryParams::<User>::from(params)
        .filter("status", Some("active"))
        .filter("email", Some("exact@match"))
        .sort("created_at", SortDirection::Descending)
        .build();

    let base_query = r#"
        SELECT u.*, d.name as department_name
        FROM users u
        LEFT JOIN departments d ON u.department_id = d.id
    "#;

    match paginated_query_as!(User, base_query)
        .with_params(params)
        .fetch_paginated(&pool)
        .await
    {
        Ok(users) => Ok(HttpResponse::Ok().json(users)),
        Err(e) => {
            log::error!("Error: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
```

## Advanced usage

### Custom base queries
```rust
let base_query = r#"
    SELECT 
        u.*,
        json_agg(r.*) as roles
    FROM users u
    LEFT JOIN user_roles ur ON u.id = ur.user_id
    LEFT JOIN roles r ON ur.role_id = r.id
    GROUP BY u.id
"#;

paginated_query_as!(User, base_query)
    .with_params(params)
    .fetch_paginated(pool)
    .await
```

### Complex filtering
```rust
let params = QueryParams::<User>::new()
    .filter("status", Some("active"))
    .filter("role", Some("admin"))
    .date_range(
        Some(Utc::now() - Duration::days(30)),
        None
    )
    .build();
```

### Extra granularity to query configuration
```rust
pub fn custom_build_query<T>(params: &QueryParams<T>) -> (Vec<String>, PgArguments)
where
    T: Default + Serialize,
{
    QueryBuilder::<T>::new()
        .add_search(params)
        .add_filters(params)
        .add_date_range(params)
        .add_raw_condition("complex SQL")
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

// Using custom builder
let users = paginated_query_as!(User, "SELECT * FROM users")
.with_params(params.clone())
.with_query_builder(custom_build_query::<User>) 
.fetch_paginated(&pool)
.await?;
```

### Error handling
```rust
match result {
    Ok(users) => HttpResponse::Ok().json(users),
    Err(sqlx::Error::RowNotFound) => {
        HttpResponse::NotFound().json(json!({
            "error": "No users found"
        }))
    }
    Err(e) => {
        log::error!("Database error: {}", e);
        HttpResponse::InternalServerError().json(json!({
            "error": "Internal server error"
        }))
    }
}
```

## Best practices

### Performance
1. Use appropriate indexes for searched columns
2. Limit page sizes
3. Use efficient base queries
4. Monitor query performance

### Security
1. Validate input parameters
2. Use type-safe filters
3. Implement proper authentication
4. Handle errors gracefully

### Maintenance
1. Keep base queries simple
2. Use proper logging
3. Monitor database performance
4. Regular testing

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
