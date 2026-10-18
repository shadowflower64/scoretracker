//! Extra file input/output functions.
//!
//! FIXME: this entire module has bad APIs and shouldn't really be used anymore.
use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::{io, result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot read file {path:?}: {e}")]
    CannotReadFile { path: PathBuf, e: io::Error },
    #[error("cannot write to file {path:?}: {e}")]
    CannotWriteFile { path: PathBuf, e: io::Error },
    #[error("cannot deserialize json {path:?}: {e}")]
    CannotDeserializeJSON { path: PathBuf, e: serde_json::Error },
    #[error("cannot serialize json {path:?}: {e}")]
    CannotSerializeJSON { path: PathBuf, e: serde_json::Error },
    #[error("cannot deserialize jsonlines {path:?}: {e}")]
    CannotDeserializeJSONLines { path: PathBuf, e: io::Error },
    #[error("cannot serialize jsonlines {path:?}: {e}")]
    CannotSerializeJSONLines { path: PathBuf, e: io::Error },
    #[error("cannot serialize toml {path:?}: {e}")]
    CannotSerializeTOML { path: PathBuf, e: toml::ser::Error },
}

impl Error {
    pub fn file_not_found(path: PathBuf) -> Self {
        Self::CannotReadFile {
            path,
            e: io::Error::from(io::ErrorKind::NotFound),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<Option<String>> {
    match fs::read_to_string(path.as_ref()) {
        Ok(content) => Ok(Some(content)),
        Err(e) => {
            if matches!(e.kind(), io::ErrorKind::NotFound) {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}

pub fn read_from_json<D: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> Result<Option<D>> {
    let path = path.as_ref();
    let content = read_to_string(path).map_err(|e| Error::CannotReadFile {
        path: path.to_path_buf(),
        e,
    })?;
    if let Some(json) = content {
        let deserialized = serde_json::from_str(&json).map_err(|e| Error::CannotDeserializeJSON {
            path: path.to_path_buf(),
            e,
        })?;
        Ok(Some(deserialized))
    } else {
        Ok(None)
    }
}

pub fn read_from_jsonlines<D: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> Result<Vec<D>> {
    let path = path.as_ref();
    serde_jsonlines::json_lines(path)
        .map_err(|e| Error::CannotReadFile {
            path: path.to_path_buf(),
            e,
        })?
        .collect::<io::Result<Vec<D>>>()
        .map_err(|e| Error::CannotDeserializeJSONLines {
            path: path.to_path_buf(),
            e,
        })
}

pub fn read_from_jsonlines_opt<D: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> Result<Option<Vec<D>>> {
    let path = path.as_ref();
    let result = serde_jsonlines::json_lines(path);
    match result {
        Err(e) => {
            if matches!(e.kind(), io::ErrorKind::NotFound) {
                Ok(None)
            } else {
                Err(Error::CannotReadFile {
                    path: path.to_path_buf(),
                    e,
                })
            }
        }
        Ok(iter) => Ok(Some(iter.collect::<io::Result<Vec<D>>>().map_err(|e| {
            Error::CannotDeserializeJSONLines {
                path: path.to_path_buf(),
                e,
            }
        })?)),
    }
}

pub fn write<C: AsRef<[u8]>>(path: impl AsRef<Path>, contents: C) -> io::Result<()> {
    fs::write(path.as_ref(), contents)
}

pub fn write_as_json<S: Serialize>(path: impl AsRef<Path>, serializable: S) -> Result<()> {
    let path = path.as_ref();
    let json = serde_json::to_string(&serializable).map_err(|e| Error::CannotSerializeJSON {
        path: path.to_path_buf(),
        e,
    })?;
    write(path, &json).map_err(|e| Error::CannotWriteFile {
        path: path.to_path_buf(),
        e,
    })?;
    Ok(())
}

pub fn write_as_json_pretty<S: Serialize>(path: impl AsRef<Path>, serializable: S) -> Result<()> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(&serializable).map_err(|e| Error::CannotSerializeJSON {
        path: path.to_path_buf(),
        e,
    })?;
    write(path, &json).map_err(|e| Error::CannotWriteFile {
        path: path.to_path_buf(),
        e,
    })?;
    Ok(())
}

pub fn write_as_jsonlines<S: Serialize>(path: impl AsRef<Path>, serializable: &[S]) -> Result<()> {
    let path = path.as_ref();
    serde_jsonlines::write_json_lines(path, serializable).map_err(|e| Error::CannotSerializeJSONLines {
        path: path.to_path_buf(),
        e,
    })?;
    Ok(())
}
