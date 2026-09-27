use serde::{Deserialize, Serialize};

use crate::{data::library::proof::GameId, sql_json_impl};

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

sql_json_impl! {ContentDescription}
