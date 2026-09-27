use dyn_clone::{DynClone, clone_trait_object};
use postgres_types::FromSql;
use serde::{Deserialize, Serialize};
use std::{any::Any, fmt::Debug};

/// This structure represents a specific set of charts in one rhythm game.
///
/// Pretty much osu!'s `beatmapset`, but for any game.
/// Often these are called "songs", but in this repo "song" means the actual music, independent of any individual rhythm game.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chartset {
    /// Game ID - which game is this chartset in?
    pub game: String,

    /// Named ID of the chartset, most often follows the convention `lowercase_artist-lowercase_title[-lowercase_chartset_version]`.
    pub chartset_id: String,

    /// Named ID of the song (music) for this chartset - often similar to the chartset ID.
    pub song_id: String,

    /// Title of the song as it appears in-game.
    pub title: String,

    /// Artist of the song as it appears in-game.
    pub artist: String,

    /// Game-specific details of the chartset.
    pub details: Box<AnyChartsetDetails>,
}

impl Chartset {
    pub fn downcast_details<T: ChartsetDetails>(&self) -> Option<&T> {
        self.details.downcast_ref()
    }
    pub fn from_postgres_row(row: &tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        Ok(Self {
            game: row.try_get("game")?,
            chartset_id: row.try_get("chartset_id")?,
            song_id: row.try_get("song_id")?,
            title: row.try_get("title")?,
            artist: row.try_get("artist")?,
            details: row.try_get("details")?,
        })
    }
}

#[typetag::serde(tag = "game")]
pub trait ChartsetDetails: Debug + DynClone + Any {
    fn game_id(&self) -> &'static str {
        self.typetag_name()
    }
}

impl dyn ChartsetDetails {
    fn any_ref(&self) -> &dyn Any {
        self
    }
    fn downcast_ref<T: ChartsetDetails>(&self) -> Option<&T> {
        self.any_ref().downcast_ref()
    }
}

clone_trait_object! {ChartsetDetails}
pub type AnyChartsetDetails = dyn ChartsetDetails;

impl<'a> FromSql<'a> for Box<AnyChartsetDetails> {
    fn from_sql(ty: &postgres_types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = serde_json::Value::from_sql(ty, raw)?;
        let details = serde_json::from_value(value)?;
        Ok(details)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        serde_json::Value::accepts(ty)
    }
}
