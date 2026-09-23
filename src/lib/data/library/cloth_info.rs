use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};

use crate::util::{timestamp::NsLocalTimestamp, uuid::UuidString};

#[derive(Debug, Clone, Serialize, Deserialize, FromSql, ToSql)]
#[postgres(name = "cloth_info")]
pub struct ClothInfo {
    /// UUID of the cloth proof file.
    pub uuid: UuidString,

    /// Start point of the cut-out video within the cloth, in nanoseconds.
    pub start_point: Option<NsLocalTimestamp>,

    /// End point of the cut-out video within the cloth, in nanoseconds.
    pub end_point: Option<NsLocalTimestamp>,
}
