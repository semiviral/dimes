use anyhow::Result;
use crate::net::Message;

pub fn unexpected_message<T>(expected: &str, actual: Message) -> Result<T> {
    bail!("Unexpcted message (expected {expected}: {actual:?}")
}
