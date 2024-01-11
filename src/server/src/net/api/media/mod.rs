use axum::Router;

mod upload;

pub fn routes() -> Router {
    Router::new().nest("/media", upload::routes())
}
