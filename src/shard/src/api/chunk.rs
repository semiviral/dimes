use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Router};
use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use uuid::Uuid;

pub fn routes() -> Router {
    Router::new().route("/chunk/:id", get(get_chunk))
}

async fn get_chunk(id: Path<Uuid>) -> impl IntoResponse {
    match crate::storage::chunk::get_chunk(&id) {
        Ok(Some(chunk)) => {
            // TODO use pooling for these responses
            let chunk_base64 = BASE64_STANDARD_NO_PAD.encode(chunk.as_slice());

            (StatusCode::OK, chunk_base64).into_response()
        }

        Ok(None) => StatusCode::NOT_FOUND.into_response(),

        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
