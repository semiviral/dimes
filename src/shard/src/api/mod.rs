use axum::{
    http::{header, HeaderValue},
    middleware::map_response,
    response::Response,
    Router,
};
use tokio::net::TcpListener;
use tower_http::{
    compression::CompressionLayer, decompression::DecompressionLayer,
    set_header::SetResponseHeaderLayer,
};

mod chunk;
mod info;

pub async fn accept_connections(listener: TcpListener) {
    trace!("Building API router...");

    let decompression_layer = DecompressionLayer::new()
        .zstd(true)
        .br(true)
        .gzip(true)
        .no_deflate();
    let compression_layer = CompressionLayer::new()
        .zstd(true)
        .br(true)
        .gzip(true)
        .no_deflate();
    let set_server_layer = SetResponseHeaderLayer::if_not_present(
        header::SERVER,
        HeaderValue::from_static(crate::agent_str()),
    );

    let app = Router::new()
        .layer(decompression_layer)
        .nest("/api", info::routes())
        .nest("/api", chunk::routes())
        .layer(set_server_layer)
        .layer(compression_layer);

    info!("Begin listening for requests.");
    axum::serve(listener, app)
        .await
        .expect("error serving connections");
}
