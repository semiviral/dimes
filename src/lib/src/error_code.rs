use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ErrorCode {
    AlreadyRegistered = 100,
    InvalidToken = 101,
}
