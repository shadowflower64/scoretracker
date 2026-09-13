use crate::data::library::stpl_url::{StplUrl, StplUrlError};
use crate::data::library::{info::LibraryInfo, library_dir_of_path, path_within_library_dir};
use crate::{config::toml::TomlConfig, util::dirs::project_temp_dir};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use uuid::Uuid;

/// A result of trying to find the info about what library a file belongs to.
///
/// Can be reused for several operations so that the library root doesn't have to be searched for every time.
pub struct LibraryRoot {
    library_dir: PathBuf,
    library_info: LibraryInfo,
}

impl LibraryRoot {
    /// Traverse the filesystem to find the library root and read the library info file.
    pub fn of(path: &Path) -> Result<Self, StplUrlError> {
        let library_dir = library_dir_of_path(&path).ok_or(StplUrlError::NotInLibraryDir)?;
        let library_info = LibraryInfo::load_from_file(path).map_err(StplUrlError::CannotReadLibraryInfo)?;
        Ok(Self { library_dir, library_info })
    }

    /// Create a StplUrl pointing to the specified file based on the results of filesystem traversal.
    ///
    /// This is a relatively cheap operation that does not read or deserialize any files, since that was already done earlier.
    pub fn url_to(&self, path: &Path) -> Result<StplUrl, StplUrlError> {
        let relpath = path_within_library_dir(&self.library_dir, path).ok_or(StplUrlError::PathFail)?;
        Ok(StplUrl::new(self.library_info.domain.clone(), Some(relpath.to_string())))
    }

    /// Return the path to the local temp directory for this library if it is defined by the [`LibraryInfo`] config.
    ///
    /// Returns [`None`] if the local temp dir is not defined.
    pub fn temp_dir(&self) -> Option<PathBuf> {
        Some(self.library_info.temp_dir.as_ref()?.to_path(&self.library_dir))
    }

    /// Return the path to the local temp directory for this library if it is defined by the [`LibraryInfo`] config.
    ///
    /// Returns the global temp dir for `scoretracker` if the local temp dir is not defined.
    pub fn temp_dir_or_default(&self) -> PathBuf {
        self.temp_dir().unwrap_or_else(project_temp_dir)
    }

    fn generate_unique_temp_filename(pretty_name: Option<&str>) -> String {
        let uuid = Uuid::now_v7();
        if let Some(pretty_name) = pretty_name {
            static INVALID_CHAR_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"[<>:"\/\\|?*]"#).expect("could not compile regex"));
            let sanitized_filename = INVALID_CHAR_REGEX.replace_all(pretty_name, "_");
            format!("{sanitized_filename}-{uuid}.tmp")
        } else {
            format!("{uuid}.tmp")
        }
    }

    /// Create a new unique temp file path for this library if a local temp directory is defined by the [`LibraryInfo`] config.
    ///
    /// Returns [`None`] if the local temp dir is not defined.
    pub fn create_temp_path(&self, pretty_name: Option<&str>) -> Option<PathBuf> {
        Some(self.temp_dir()?.join(Self::generate_unique_temp_filename(pretty_name)))
    }

    /// Create a new unique temp file path for this library.
    ///
    /// This function returns a path in a local temp directory if it is defined by the [`LibraryInfo`] config.
    /// If it is not defined, it falls back to a unique path in the global temp dir for `scoretracker`.
    pub fn create_temp_path_or_default(&self, pretty_name: Option<&str>) -> PathBuf {
        self.temp_dir_or_default().join(Self::generate_unique_temp_filename(pretty_name))
    }
}
