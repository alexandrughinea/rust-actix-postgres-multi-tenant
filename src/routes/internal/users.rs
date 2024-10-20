use crate::configurations::Configuration;
use crate::models::{AppState, User};
use crate::utils::{get_pool_for_tenant, get_tenant_id_from_request};
use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;
use sqlx::PgPool;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(create_user))
        .route("", web::get().to(get_users));
}

pub async fn get_users(
    req: HttpRequest,
    state: web::Data<AppState>,
    pool: web::Data<PgPool>,
    settings: web::Data<Configuration>,
) -> HttpResponse {
    let tenant_id = match get_tenant_id_from_request(&req) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let tenant_pool = match get_pool_for_tenant(&tenant_id, &state, &pool, &settings).await {
        Ok(pool) => pool,
        Err(e) => return e,
    };

    let users = match sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(tenant_pool.as_ref())
        .await
    {
        Ok(users) => users,
        Err(_) => return HttpResponse::InternalServerError().body("Error fetching users"),
    };

    HttpResponse::Ok().json(json!(users))
}

pub async fn create_user(
    req: HttpRequest,
    user: web::Json<User>,
    state: web::Data<AppState>,
    pool: web::Data<PgPool>,
    settings: web::Data<Configuration>,
) -> HttpResponse {
    let tenant_id = match get_tenant_id_from_request(&req) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let pool = match get_pool_for_tenant(&tenant_id, &state, &pool, &settings).await {
        Ok(pool) => pool,
        Err(e) => return e,
    };

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (tenant_id, name)
        VALUES ($1, $2)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&user.name)
    .fetch_one(pool.as_ref())
    .await
    .unwrap();

    HttpResponse::Ok().json(json!(user))
}
