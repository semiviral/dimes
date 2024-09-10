use crate::storage::chunk::chunk_exists;
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use lib::chunk::Chunk;
use uuid::Uuid;

pub fn routes() -> Router {
    Router::new()
        .route("/chunk/:id", get(get_chunk).put(put_chunk))
        .layer(DefaultBodyLimit::max(Chunk::SIZE))
}

async fn get_chunk(id: Path<Uuid>) -> impl IntoResponse {
    match crate::storage::chunk::get_chunk(*id).await {
        Ok(Some(chunk)) => {
            trace!("Serving chunk: {} / {:X?}", chunk.id(), &chunk[..16]);

            (StatusCode::OK, chunk).into_response()
        }

        Ok(None) => StatusCode::NOT_FOUND.into_response(),

        Err(err) => {
            error!("Error handling request: {err:?}");

            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn put_chunk(id: Path<Uuid>, body: Bytes) -> impl IntoResponse {
    // Ensure the response body is the correct size.
    if body.len() != Chunk::SIZE {
        return StatusCode::BAD_REQUEST.into_response();
    }

    match chunk_exists(*id) {
        Ok(false) => {
            debug!("Received request to insert chunk: {}", *id);
        }

        Ok(true) => return StatusCode::CONFLICT.into_response(),

        Err(err) => {
            error!("Error checking if chunk exists: {}\n{err:?}", *id);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }

    trace!("Inserting chunk: {} / {:X?}", *id, &body[..12]);

    let mut chunk = Chunk::new_zeroed(*id).await;
    chunk.copy_from_slice(&body);

    match crate::storage::chunk::put_chunk(chunk) {
        Ok(_) => {
            trace!("Inserted chunk: {}", *id);

            StatusCode::CREATED.into_response()
        }

        Err(err) => {
            error!("Error inserting chunk: {}\n{err:?}", *id);

            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
