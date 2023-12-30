use crate::Token;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Upload {
    token: Token,
    data: Box<[u8]>,
}
