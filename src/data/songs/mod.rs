use crate::data::songs::song::SongTrait;
use serde::{Deserialize, Serialize};

pub mod song;

#[derive(Deserialize, Serialize)]
pub struct GameSongList<Song: SongTrait> {
    pub format_version: i32,
    pub game_id: String,
    pub songs: Vec<Song>,
}
