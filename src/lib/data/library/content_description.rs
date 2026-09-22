use std::error::Error;

use postgres_types::{FromSql, IsNull, ToSql, to_sql_checked};
use serde::{Deserialize, Serialize};

use crate::data::library::entry::GameId;

/// The contents of the video or image that the library entry is associated with - what kind of footage does the video show?
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case", tag = "category")]
pub enum ContentDescription {
    /// Default value - value not selected by user yet.
    #[default]
    Unspecified,

    /// The video shows the song select screen, the entire playthrough of one song, and the end screen.
    GameplayNormal { game: Option<GameId> },

    /// The video or image shows only gameplay, and does not show the score screen at the end.
    GameplayOnly { game: Option<GameId> },

    /// The video or image shows only the results screen, and does not show the gameplay.
    ResultsScreen { game: Option<GameId> },

    /// The video or image depicts some part of the game, but the contents of the video or image don't belong to any other more specific category.
    GameGeneric { game: Option<GameId> },

    /// The contents of the video or image don't belong to any other category.
    Other { description: Option<String> },
}

impl<'a> FromSql<'a> for ContentDescription {
    fn accepts(ty: &postgres_types::Type) -> bool {
        <serde_json::Value as FromSql>::accepts(ty)
    }
    fn from_sql(ty: &postgres_types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = serde_json::Value::from_sql(ty, raw)?;
        Ok(serde_json::from_value(value)?)
    }
}

impl ToSql for ContentDescription {
    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        <serde_json::Value as ToSql>::accepts(ty)
    }
    fn to_sql(&self, ty: &postgres_types::Type, out: &mut actix_web::web::BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        serde_json::value::to_value(self.clone())?.to_sql(ty, out)
    }
    to_sql_checked! {}
}
