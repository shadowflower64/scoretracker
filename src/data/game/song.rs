use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SongAlbumInfo {
    Single,
    Album { name: String },
}

impl SongAlbumInfo {
    pub fn album_name(self) -> Option<String> {
        match self {
            Self::Album { name } => Some(name),
            Self::Single => None,
        }
    }
    pub fn is_album(&self) -> bool {
        matches!(self, Self::Album { .. })
    }
    pub fn is_single(&self) -> bool {
        matches!(self, Self::Single)
    }
}

impl From<SongAlbumInfo> for Option<String> {
    fn from(value: SongAlbumInfo) -> Self {
        value.album_name()
    }
}

pub trait SongTrait: Debug {
    fn global_song_id(&self) -> Option<Uuid> {
        None
    }
    fn title(&self) -> String;
    fn artist(&self) -> String;
    fn album(&self) -> Option<SongAlbumInfo> {
        None
    }
    fn year(&self) -> Option<i64> {
        None
    }
}

pub type AnySong = Box<dyn SongTrait>;
