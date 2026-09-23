use std::{
    fmt::{self, Display},
    hash::Hash,
};

use postgres_types::{FromSql, ToSql, to_sql_checked};
use serde::{Deserialize, Serialize, de::Visitor};

/// This struct represents a SHA256 hash digest. It holds exactly 32 bytes.
///
/// ## Serialization
///
/// For JSON, this struct gets (de-)serialized as a 64-character lowercase hexadecimal string.
///
/// In the database, this struct gets represented as a `bytea` (unsized byte array) field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sha256Hash(pub [u8; 32]);

impl Serialize for Sha256Hash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl Display for Sha256Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Sha256Hash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Sha256Hash;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a lowercase hex sha256 hash string")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                let mut buf = [0u8; 32];
                hex::decode_to_slice(v, &mut buf).unwrap(); // TODO: handle invalid length
                Ok(Sha256Hash(buf))
            }

            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_str(V)
    }
}

impl<'a> FromSql<'a> for Sha256Hash {
    fn accepts(ty: &postgres_types::Type) -> bool {
        <Vec<u8> as FromSql>::accepts(ty)
    }
    fn from_sql(ty: &postgres_types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let vec: Vec<u8> = Vec::from_sql(ty, raw)?;
        let mut buf = [0u8; 32];
        buf.clone_from_slice(&vec); // TODO: handle invalid length
        Ok(Self(buf))
    }
}

impl ToSql for Sha256Hash {
    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <Vec<u8> as ToSql>::accepts(ty)
    }
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut actix_web::web::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let mut vec = Vec::with_capacity(self.0.len());
        vec.extend_from_slice(&self.0);
        vec.to_sql(ty, out)
    }
    to_sql_checked! {}
}
