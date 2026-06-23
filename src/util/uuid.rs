//! Module for [`UuidString`], a (de)serializable wrapper for [`Uuid`].
use serde::{Deserialize, Serialize, de::Visitor};
use std::fmt::{self, Display};
use std::str::FromStr;
use uuid::Uuid;

/// Wrapper structure for serializing and deserializing UUID values.
///
/// This structure uses the [`Uuid::from_str`] function to deserialize string values into UUIDs,
/// and uses the [`ToString::to_string`] function to serialize UUID values into strings.
///
/// This struct should be used for UUID fields within (de)serializable structs, but it shouldn't be otherwise used over the usual [`Uuid`] struct.
///
/// You can convert between [`Uuid`] and [`UuidString`] easily by using `.into()`,
/// or by reading the `.0` field of this struct,
/// or by constructing this struct directly like this: `UuidString(uuid)`.
///
/// # Example
/// ```
/// use uuid::Uuid;
/// use scoretracker::util::uuid::UuidString;
///
/// let uuid = Uuid::now_v7();
/// let uuid_string = UuidString::from(uuid);
/// assert_eq!(uuid_string.0, uuid);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UuidString(pub Uuid);

impl From<Uuid> for UuidString {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<UuidString> for Uuid {
    fn from(value: UuidString) -> Self {
        value.0
    }
}

impl Serialize for UuidString {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Display for UuidString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

struct UuidVisitor;

impl<'de> Visitor<'de> for UuidVisitor {
    type Value = UuidString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a uuid string")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(UuidString(Uuid::from_str(v).unwrap()))
    }

    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
        self.visit_str(&v)
    }
}

impl<'de> Deserialize<'de> for UuidString {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(UuidVisitor)
    }
}
