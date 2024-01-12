use axum::Router;

mod register;

pub fn routes() -> Router {
    Router::new().nest("/shard", register::routes())
}
