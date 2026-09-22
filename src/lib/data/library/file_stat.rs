use std::{
    collections::HashMap,
    error::Error,
    fs,
    ops::{Deref, DerefMut},
    path::Path,
};

use postgres_types::{FromSql, IsNull, ToSql, to_sql_checked};
use serde::{Deserialize, Serialize};

use crate::{data::library::stpl_url::StplUrl, util::timestamp::NsTimestamp};

/// Basic metadata about the file from the `stat` command.
///
/// This struct stores basic metadata about the file, such as the file's size, the file modification time, and the file creation time.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FileStat {
    /// Size of the file, in bytes.
    pub size: u64,

    /// Birth of the file - when was this file created on the disk?
    ///
    /// For raw video files, this is usually the time when the video has started recording.
    pub timestamp_birth: NsTimestamp,

    /// Access of the file - when was this file last accessed or read?
    pub timestamp_access: NsTimestamp,

    /// Modification - when was the data inside of this file modified? For raw video files, this is usually the time when the video has finished recording.
    ///
    /// This value may be set by tools such as LosslessCut to indicate a video recording timestamp, however it may be wrong.
    /// I think LosslessCut actually moves the timestamp wrongly.
    pub timestamp_modification: NsTimestamp,

    /// Status change - when were the permissions(?) changed for this file?
    //pub timestamp_status_change: NsTimestamp,
    // this doesn't seem to actually work cross-platform/it's not in the rust api.

    /// Timestamp of when was the file stat was read (This is not actually part of the `stat` command, and it is stored manually.)
    pub last_check: NsTimestamp,
}

impl FileStat {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let metadata = fs::metadata(path).unwrap();
        let size = metadata.len();
        let timestamp_birth = metadata.created().unwrap().into();
        let timestamp_access = metadata.accessed().unwrap().into();
        let timestamp_modification = metadata.modified().unwrap().into();

        FileStat {
            size,
            timestamp_birth,
            timestamp_access,
            timestamp_modification,
            last_check: NsTimestamp::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct FileStats(pub HashMap<StplUrl, FileStat>);

impl FileStats {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl Deref for FileStats {
    type Target = HashMap<StplUrl, FileStat>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FileStats {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> FromSql<'a> for FileStats {
    fn accepts(ty: &postgres_types::Type) -> bool {
        <serde_json::Value as FromSql>::accepts(ty)
    }
    fn from_sql(ty: &postgres_types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = serde_json::Value::from_sql(ty, raw)?;
        Ok(Self(serde_json::from_value(value)?))
    }
}

impl ToSql for FileStats {
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
        serde_json::value::to_value(self.0.clone())?.to_sql(ty, out)
    }
    to_sql_checked! {}
}
