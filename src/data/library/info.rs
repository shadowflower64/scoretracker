//! Library info file handling.
//!
//! A "library info file" is a file that contains basic information about the library.
//! For example, this file contains the domain name for this library dir.
use crate::data::library::stpl_url::LibraryDomain;
use crate::util::file_ex::FileEx;
use crate::util::{file_ex, filelocked::FileLockableData};
use serde::{Deserialize, Serialize};

/// Basic info about the library.
///
/// This structure contains information about the library. Currently, it only contains the domain name.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LibraryInfo {
    pub domain: LibraryDomain,
}

impl LibraryInfo {
    pub const STANDARD_FILENAME: &str = "library_info.json";
}

impl FileLockableData for LibraryInfo {
    fn _inner_read<F: FileEx + ?Sized>(file_ex: &F) -> file_ex::Result<Option<Self>> {
        file_ex.read_from_json()
    }

    fn _inner_write<F: FileEx + ?Sized>(&self, file_ex: &F) -> file_ex::Result<()> {
        file_ex.write_as_json_pretty(self)
    }
}
