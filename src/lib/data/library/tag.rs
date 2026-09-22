use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use postgres_types::{FromSql, ToSql, to_sql_checked};
use serde::{Deserialize, Serialize};

pub type Tag = String;

/// Wrapper struct for (de-)serializing into database types with FromSql and ToSql.
///
/// This is a wrapper for [`HashSet<Tag>`]. Contained values are unique and unordered.
///
/// ## Serialization
///
/// When serialized to JSON, or when stored in the database, this struct is represented as an array of strings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tags(pub HashSet<Tag>);

impl Tags {
    pub fn new() -> Self {
        Self(HashSet::new())
    }
}

impl Deref for Tags {
    type Target = HashSet<Tag>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Tags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> FromSql<'a> for Tags {
    fn accepts(ty: &postgres_types::Type) -> bool {
        <Vec<Tag> as FromSql>::accepts(ty)
    }
    fn from_sql(ty: &postgres_types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let vec: Vec<Tag> = Vec::from_sql(ty, raw)?;
        Ok(Self(HashSet::from_iter(vec)))
    }
}

impl ToSql for Tags {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut actix_web::web::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let vec: Vec<&Tag> = self.0.iter().collect();
        vec.to_sql(ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        <Vec<Tag> as ToSql>::accepts(ty)
    }

    to_sql_checked! {}
}
