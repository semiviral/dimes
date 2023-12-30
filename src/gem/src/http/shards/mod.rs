use axum::Router;

pub mod register;

pub fn routes() -> Router {
    Router::new().nest("/shards", register::routes())
}
