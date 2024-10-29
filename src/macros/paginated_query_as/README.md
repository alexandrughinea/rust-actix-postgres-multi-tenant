# Paginated Query Builder for SQLx PostgreSQL

A flexible, type-safe query builder for handling paginated queries in Rust with SQLx and PostgreSQL. Built for robust handling of dynamic web API queries with built-in support for pagination, searching, filtering, and sorting.

## Table of Contents
- [Features](#features)
- [Why Use This Library](#why-use-this-library)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Documentation](#api-documentation)
- [Framework Integration](#framework-integration)
- [Advanced Usage](#advanced-usage)
- [Best Practices](#best-practices)
- [Contributing](#contributing)
- [License](#license)

## Features

### Core Capabilities
- 🔍 Full-text search with column specification
- 📑 Smart pagination with customizable page size
- 🔄 Dynamic sorting on any column
- 🎯 Flexible filtering system
- 📅 Date range filtering
- 🔒 Type-safe operations
- ⚡ High performance
- 🛡️ SQL injection protection

### Technical Features
- Builder pattern for query construction
- Automatic PostgreSQL type casting
- Connection pool management
- Error handling and logging
- Serialization/deserialization support
- Custom base query support
- Multi-tenancy capabilities

### Query Features
- Case-insensitive search
- Multiple column search
- Complex filtering conditions
- Date-based filtering
- Dynamic sort direction
- Customizable page size
- Result count optimization

## Why Use This Library

### Problems Solved
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

### Compared to Alternatives
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

## Quick Start

```rust
use your_crate::paginated_query_as;

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

## API Documentation

### QueryParams Builder
```rust
pub struct QueryParams<T> {
    pub pagination: PaginationParams,
    pub sort: SortParams,
    pub search: SearchParams,
    pub date_range: DateRangeParams,
    pub filters: HashMap<String, Option<String>>,
}
```

### Methods

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

#### Date Range
```rust
/// Set date range for filtering
fn date_range(after: Option<DateTime<Utc>>, before: Option<DateTime<Utc>>) -> Self

// Example
.date_range(Some(Utc::now() - Duration::days(7)), None)
```

### Response Type
```rust
pub struct PaginatedResponse<T> {
    pub records: Vec<T>,
    pub total: i64,
    pub pagination: PaginationParams,
    pub total_pages: i64,
}
```

### Query Parameters from HTTP
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

### Response Format
```json
{
  "records": [...],
  "total": 100,
  "page": 1,
  "page_size": 10,
  "total_pages": 10
}
```

## Framework Integration

### Actix Web Integration

#### Setup
```toml
[dependencies]
actix-web = "4.0"
```

#### Basic Handler
```rust
use actix_web::{web, HttpResponse};

pub async fn get_users(
    pool: web::Data<PgPool>,
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

#### Advanced Handler with Custom Logic
```rust
pub async fn get_users_custom(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    Query(params): Query<FlatQueryParams>,
) -> Result<HttpResponse, Error> {
    let params = QueryParams::<User>::from(params)
        .filter("status", Some("active"))
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

#### App Configuration
```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://localhost/db")
        .await
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api")
                    .route("/users", web::get().to(get_users))
                    .route("/users/custom", web::get().to(get_users_custom))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Advanced Usage

### Custom Base Queries
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

### Complex Filtering
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

### Error Handling
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

## Best Practices

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
