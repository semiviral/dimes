use crate::token::{Server, Token};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Upload {
    token: Token<Server>,
    data: Box<[u8]>,
}
