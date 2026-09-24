use dyn_clone::{DynClone, clone_trait_object};
use std::fmt::Debug;

/// This structure represents a specific set of charts in one rhythm game.
///
/// Pretty much osu!'s `beatmapset`, but for any game.
/// Often these are called "songs", but in this repo "song" means the actual music, independent of any individual rhythm game.
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

#[typetag::serde(tag = "game")]
pub trait ChartsetDetails: Debug + DynClone {
    fn game_id(&self) -> &'static str {
        self.typetag_name()
    }
}

clone_trait_object! {ChartsetDetails}
pub type AnyChartsetDetails = dyn ChartsetDetails + 'static;
