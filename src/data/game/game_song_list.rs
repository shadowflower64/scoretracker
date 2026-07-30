use crate::data::game::song::SongTrait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GameSongList<Song: SongTrait> {
    pub format_version: i32,
    pub game_id: String,
    pub songs: Vec<Song>,
}
