use axum::Router;

mod multipart;
mod resumable;

pub fn routes() -> Router {
    Router::new()
        .nest("/upload", multipart::routes())
        .nest("/upload", resumable::routes())
}
