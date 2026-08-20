//! Library auxililary data file handling.
//!
//! A "library auxiliary data file" is a file that contains additional information about the library that is not the actual files of the library.
//! For example, auxiliary data may contain information about the library's tags.
use crate::util::filelocked::FileLockableDataJson;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct ColorRGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TagInfo {
    pub id: String,
    pub name: String,
    pub color: ColorRGBA,
}

/// Auxiliary data for the library.
///
/// This structure contains information on various additional aspects of the library, that are not proof entries.
/// For example, it contains information about the names and colors of tags used in the library.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LibraryAuxData {
    pub tags: Vec<TagInfo>,
}

impl LibraryAuxData {
    pub const STANDARD_FILENAME: &str = "library_aux.json";
}

impl FileLockableDataJson for LibraryAuxData {}
