use crate::util::{
    file_ex::{self, FileEx},
    filelocked::FileLockableData,
    relative_path_from_segments,
    uuid::UuidString,
};
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Player {
    pub uuid: UuidString,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct PlayerDatabase {
    pub players: Vec<Player>,
}

impl PlayerDatabase {
    pub const STANDARD_PATH_SEGMENTS: [&str; 2] = ["data", "players.jsonl"];

    pub fn path_within_shared_repo() -> &'static RelativePath {
        static CACHE: LazyLock<RelativePathBuf> = LazyLock::new(|| relative_path_from_segments(&PlayerDatabase::STANDARD_PATH_SEGMENTS));
        &CACHE
    }

    pub fn find_player_by_name(&self, name: &str) -> Option<&Player> {
        self.players.iter().find(|x| x.name == name)
    }

    pub fn find_player_by_uuid(&self, uuid: UuidString) -> Option<&Player> {
        self.players.iter().find(|x| x.uuid == uuid)
    }

    pub fn find_player_by_uuid_mut(&mut self, uuid: UuidString) -> Option<&mut Player> {
        self.players.iter_mut().find(|x| x.uuid == uuid)
    }

    pub fn add(&mut self, name: &str) -> Result<Uuid, Uuid> {
        if let Some(old_player) = self.find_player_by_name(name) {
            Err(old_player.uuid.into())
        } else {
            let uuid = Uuid::now_v7();
            self.players.push(Player {
                uuid: uuid.into(),
                name: name.to_owned(),
            });
            Ok(uuid)
        }
    }
}

impl FileLockableData for PlayerDatabase {
    fn _inner_read<F: FileEx + ?Sized>(file_ex: &F) -> file_ex::Result<Option<Self>> {
        file_ex.read_from_jsonlines().map(|x| x.map(|y| Self { players: y }))
    }
    fn _inner_write<F: FileEx + ?Sized>(&self, file_ex: &F) -> file_ex::Result<()> {
        file_ex.write_as_jsonlines(&self.players)
    }
}
