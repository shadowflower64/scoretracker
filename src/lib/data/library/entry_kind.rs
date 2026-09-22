use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};

/// Kind of the library entry - is it a proof of a performance or something else?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, FromSql, ToSql)]
#[serde(rename_all = "snake_case")]
#[postgres(rename_all = "snake_case")]
pub enum LibraryEntryKind {
    /// Default value - value not selected by user yet.
    #[default]
    #[serde(alias = "unset")] // temp alias for migration while testing, can be removed later (TODO)
    Unspecified,

    /// Video not showing a performance, unrelated to proof stuff but still in library for some reason.
    NotProof,

    /// Video showing a performance, but not yet possible to associate with a performance - the performance is not saveable in database for some reason. for example, one-finger-challenge FCs.
    Unsupported,

    /// Video showing a performance, but not yet associated with a performance.
    NotLinkedYet,

    /// Video showing a performance, associated with a performance or multiple performances.
    Linked,
}
