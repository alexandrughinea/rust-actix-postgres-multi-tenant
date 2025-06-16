use crate::configurations::Configuration;
use crate::models::AppState;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct Product {
    id: i32,
    name: String,
    description: String,
    price: f64,
    stock: i32,
}

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_products))
        .route("", web::post().to(create_product))
        .route("/{id}", web::get().to(get_product));
}

pub async fn get_products(
    _req: HttpRequest,
    _state: web::Data<AppState>,
    _pool: web::Data<PgPool>,
    _configuration: web::Data<Configuration>,
) -> HttpResponse {
    // Dummy response with multiple products
    let products = vec![
        Product {
            id: 1,
            name: "Product 1".to_string(),
            description: "Description for product 1".to_string(),
            price: 19.99,
            stock: 100,
        },
        Product {
            id: 2,
            name: "Product 2".to_string(),
            description: "Description for product 2".to_string(),
            price: 29.99,
            stock: 50,
        },
        Product {
            id: 3,
            name: "Product 3".to_string(),
            description: "Description for product 3".to_string(),
            price: 39.99,
            stock: 75,
        },
    ];

    HttpResponse::Ok().json(json!({
        "data": products,
        "metadata": {
            "total": 3,
            "page": 1,
            "per_page": 10
        }
    }))
}

pub async fn create_product(
    _req: HttpRequest,
    product: web::Json<Product>,
    _state: web::Data<AppState>,
    _pool: web::Data<PgPool>,
    _configuration: web::Data<Configuration>,
) -> HttpResponse {
    // Dummy response for creating a product
    let new_product = Product {
        id: 4, // Pretend we created with a new ID
        name: product.name.clone(),
        description: product.description.clone(),
        price: product.price,
        stock: product.stock,
    };

    HttpResponse::Created().json(json!(new_product))
}

pub async fn get_product(
    _req: HttpRequest,
    path: web::Path<i32>,
    _state: web::Data<AppState>,
    _pool: web::Data<PgPool>,
    _configuration: web::Data<Configuration>,
) -> HttpResponse {
    let id = path.into_inner();

    // Dummy response for a single product
    let product = Product {
        id,
        name: format!("Product {}", id),
        description: format!("Description for product {}", id),
        price: 19.99 * (id as f64),
        stock: 100 - (id * 10),
    };

    HttpResponse::Ok().json(json!(product))
}
