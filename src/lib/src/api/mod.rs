use anyhow::Result;
use reqwest::Response;
use serde::de::DeserializeOwned;

pub mod server;
pub mod shard;

pub async fn response_json<T: DeserializeOwned>(response: Response) -> Result<T> {
    let response = response.error_for_status()?;
    let content_type = response
        .headers()
        .get("Content-Type")
        .ok_or(anyhow!("response does not have a `Content-Type` header"))?;

    if content_type == "application/json" {
        Ok(response.json().await?)
    } else {
        bail!("Content-Type header is not `application/json`")
    }
}
