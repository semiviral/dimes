use axum::{
    body::{Body, Bytes},
    extract::Path,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use redis::AsyncCommands;
use serde::Deserialize;

pub fn add_routes(router: Router) -> Router {
    router.route("/chunks/:id", get(get_chunk).put(put_chunk))
}

async fn get_chunk(Path(id): Path<u64>) -> impl IntoResponse {
    trace!("Serving request for chunk #{}", id);

    let mut guard = crate::REDIS_CONN.lock().await;
    let redis_conn = guard.get_mut().unwrap();

    let result = redis_conn.hget::<_, _, Option<String>>(id, "blob").await;
    trace!("Request result: {result:?}");

    match result {
        Err(err) => panic!("{err:?}"),

        Ok(None) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),

        Ok(blob) => Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(blob.unwrap()))
            .unwrap(),
    }
}

async fn put_chunk(Path(id): Path<u64>, body: Bytes) -> impl IntoResponse {
    
}
