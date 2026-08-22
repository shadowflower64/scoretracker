use serde::Deserialize;
use std::{fs, io, path::Path};
use thiserror::Error;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    /// In seconds since the beginning of the source video.
    pub start: f64,

    /// In seconds since the beginning of the source video.
    pub end: f64,

    /// Optional; empty string if no name was set.
    pub name: String,

    pub selected: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlcProj {
    pub version: u32,
    pub media_file_name: String,
    pub cut_segments: Vec<Segment>,
}

#[derive(Debug, Error)]
pub enum LlcProjError {
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
    #[error("yaml error: {0}")]
    YamlError(#[from] yaml_serde::Error),
}

impl LlcProj {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, LlcProjError> {
        let file_contents = fs::read_to_string(path)?;
        let llc_proj = yaml_serde::from_str(&file_contents)?;
        Ok(llc_proj)
    }
}
