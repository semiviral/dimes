use axum::Router;

pub fn routes() -> Router {
    Router::new().route("/chunk", todo!())
}
