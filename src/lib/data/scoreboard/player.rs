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
