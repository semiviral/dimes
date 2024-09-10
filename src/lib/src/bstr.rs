use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BStr<const LEN: usize> {
    #[serde(with = "serde_arrays")]
    bytes: [u8; LEN],
    str_len: usize,
}

impl<const LEN: usize> BStr<LEN> {
    pub fn new(s: impl AsRef<str>) -> Self {
        let s = s.as_ref();
        let str_len = s.len();

        assert!(str_len <= LEN);

        let mut bytes = [0u8; LEN];
        bytes[..s.len()].copy_from_slice(s.as_bytes());

        Self { bytes, str_len }
    }
}

impl<const LEN: usize> std::ops::Deref for BStr<LEN> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // SAFETY: `bytes` is copied from an existing `&str`.
        unsafe { std::str::from_utf8_unchecked(&self.bytes[..self.str_len]) }
    }
}
