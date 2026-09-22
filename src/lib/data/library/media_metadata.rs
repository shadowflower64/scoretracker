use std::{
    collections::HashMap,
    error::Error,
    ops::{Deref, DerefMut},
};

use actix_web::web::BytesMut;
use postgres_types::{FromSql, IsNull, ToSql, to_sql_checked};
use serde::{Deserialize, Serialize};

/// This type represents the internal metadata of a media file (creation_date, android version, video/audio stream count, other similar metadata).
///
/// This may store information such as EXIF information of a file for easier lookups.
/// Currently it is always empty; no part of the code actually reads that kind of metadata.
///
/// The structure format is subject to change. (TODO: maybe it should be a HashMap<String, serde_json::Value>? or maybe even an IndexMap?)
///
/// ## Serialization
/// (De-)serialization to JSON is pretty self-explanatory.
///
/// As for (de-)serialization in the context of a database, the database stores the value as a `jsonb` type.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct MediaMetadata(pub HashMap<String, String>);

impl Deref for MediaMetadata {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MediaMetadata {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> FromSql<'a> for MediaMetadata {
    fn accepts(ty: &postgres_types::Type) -> bool {
        <serde_json::Value as FromSql>::accepts(ty)
    }
    fn from_sql(ty: &postgres_types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = serde_json::Value::from_sql(ty, raw)?;
        Ok(Self(serde_json::from_value(value)?))
    }
}

impl ToSql for MediaMetadata {
    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <serde_json::Value as ToSql>::accepts(ty)
    }
    fn to_sql(&self, ty: &postgres_types::Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        serde_json::value::to_value(self.0.clone())?.to_sql(ty, out)
    }
    to_sql_checked! {}
}
