use actix_web::web;
mod products;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/products").configure(products::configure));
}
