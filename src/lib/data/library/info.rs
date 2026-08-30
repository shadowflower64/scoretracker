//! Library info file handling.
//!
//! A "library info file" is a file that contains basic information about the library.
//! For example, this file contains the domain name for this library dir.
use crate::data::library::stpl_url::LibraryDomain;
use crate::util::file_ex::FileEx;
use crate::util::{file_ex, filelocked::FileLockableData};
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};

/// Basic info about the library.
///
/// This structure contains information about the library repository at this file's location.
/// The presence of the library_info.json file in the directory indicates that the directory is hosting a library repository.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LibraryInfo {
    /// Domain name for this library.
    ///
    /// This name is the
    pub domain: LibraryDomain,

    /// Relative path to the directory for temporary files related to this library.
    ///
    /// This path is stored as a relative path, because the filesystem that this library repository lives on
    /// may be mounted on different mountpoints (and even different operating systems!) at various times.
    /// If this library should not have a local temp dir, and a global temp dir should be used instead,
    /// the option may be set to [`None`].
    ///
    /// Workers will use this directory for example for storing ffmpeg output files.
    /// The advantage of using this over the normal system-wide temp directory is that after finishing the process,
    /// the file does not need to be copied, only moved, as it is ideally on the same filesystem as the rest of the library.
    pub temp_dir: Option<RelativePathBuf>,
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
