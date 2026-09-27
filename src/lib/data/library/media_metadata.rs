use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::sql_json_impl;

/// This type represents the internal metadata of a media file (creation_date, android version, video/audio stream count, other similar metadata).
///
/// This may store information such as EXIF information of a file for easier lookups.
/// Currently it is always empty; no part of the code actually reads that kind of metadata.
///
/// The structure format is subject to change. (TODO: maybe it should be a HashMap<String, serde_json::Value>? or maybe even an IndexMap?)
///
/// ## Serialization
/// (De-)serialization to JSON is pretty self-explanatory.
///
/// As for (de-)serialization in the context of a database, the database stores the value as a `jsonb` type.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct MediaMetadata(pub HashMap<String, String>);

impl Deref for MediaMetadata {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MediaMetadata {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

sql_json_impl! {MediaMetadata}
