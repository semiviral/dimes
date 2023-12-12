mod chunks;

use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::options,
    Router,
};

pub fn make_router() -> Router {
    let mut router = Router::new();

    router = add_routes(router);
    router = chunks::add_routes( router);

    router
}

fn add_routes(router: Router) -> Router {
    router.route(
        "/",
        options(|| async {
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap()
        }),
    ).route("/health", options(health_check).get(health_check))
}

async fn health_check() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}
