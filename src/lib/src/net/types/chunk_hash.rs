#[repr(transparent)]
#[derive(
    Debug, serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct ChunkHash([u8; 16]);
