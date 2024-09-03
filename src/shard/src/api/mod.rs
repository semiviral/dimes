use axum::{middleware::map_response, response::Response, Router};
use tokio::net::TcpListener;

mod chunk;
mod info;

pub async fn accept_connections(listener: TcpListener) {
    async fn add_default_headers<B>(mut response: Response<B>) -> Response<B> {
        use axum::http::header;

        let headers = response.headers_mut();
        headers.insert(header::SERVER, crate::agent_str().parse().unwrap());
        headers.insert(
            header::DATE,
            chrono::Utc::now().to_rfc3339().parse().unwrap(),
        );

        response
    }

    trace!("Building API router...");
    let app = Router::new()
        .nest("/api", info::routes())
        .layer(map_response(add_default_headers));

    info!("Begin listening for requests.");
    axum::serve(listener, app)
        .await
        .expect("error serving connections");
}
