use actix_web::web;
mod users;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/users").configure(users::configure));
}
