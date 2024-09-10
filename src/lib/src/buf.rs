use std::io::{Cursor, Read, Write};

use uuid::Uuid;

macro_rules! pop_type_le {
    ($ty:ty) => {
        paste::paste! {
            pub fn [<pop_ $ty _le>](&mut self) -> Result<$ty> {
                let mut buf = [0u8; size_of::<$ty>()];
                self.0.read_exact(&mut buf)?;

                Ok($ty::from_le_bytes(buf))
            }
        }
    };
}

macro_rules! push_type_le {
    ($ty:ty) => {
        paste::paste! {
            pub fn [<push_ $ty _le>](&mut self, value: $ty) {
                self.0.write_all(&value.to_le_bytes());
            }
        }
    };
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("buffer is empty")]
    Empty,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::UnexpectedEof => Self::Empty,

            _ => unimplemented!(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Buf(Cursor<Vec<u8>>);

impl Buf {
    pub const fn new() -> Self {
        Self(Cursor::new(Vec::new()))
    }

    pop_type_le!(u8);
    pop_type_le!(u16);
    pop_type_le!(u32);
    pop_type_le!(u64);
    pop_type_le!(u128);

    pop_type_le!(i8);
    pop_type_le!(i16);
    pop_type_le!(i32);
    pop_type_le!(i64);
    pop_type_le!(i128);

    push_type_le!(u8);
    push_type_le!(u16);
    push_type_le!(u32);
    push_type_le!(u64);
    push_type_le!(u128);

    push_type_le!(i8);
    push_type_le!(i16);
    push_type_le!(i32);
    push_type_le!(i64);
    push_type_le!(i128);

    pub fn pop_uuid_le(&mut self) -> Result<Uuid> {
        let mut buf = [0u8; size_of::<Uuid>()];
        self.0.read_exact(&mut buf)?;

        Ok(Uuid::from_bytes_le(buf))
    }

    pub fn push_uuid_le(&mut self, uuid: Uuid) {
        self.0.write_all(&uuid.to_bytes_le());
    }

    pub fn pop(&mut self, buf: &mut [u8]) -> Result<()> {
        self.0.read_exact(buf)?;

        Ok(())
    }

    pub fn push(&mut self, data: &[u8]) {
        self.0.write_all(data);
    }
}

impl<T: AsMut<Vec<u8>>> From<T> for Buf {
    fn from(value: T) -> Self {
        Self(Cursor::new(value.as_mut()))
    }
}

impl AsRef<[u8]> for Buf {
    fn as_ref(&self) -> &[u8] {
        self.0.get_ref().as_slice()
    }
}

impl AsMut<[u8]> for Buf {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.get_mut().as_mut_slice()
    }
}

impl chacha20poly1305::aead::Buffer for Buf {
    fn extend_from_slice(&mut self, other: &[u8]) -> chacha20poly1305::aead::Result<()> {
        self.0.write_all(other);

        Ok(())
    }

    fn truncate(&mut self, len: usize) {
        self.0.get_mut().truncate(len);

        let len = len.try_into().unwrap();
        if len > self.0.position() {
            self.0.set_position(len);
        }
    }
}
