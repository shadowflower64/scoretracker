use crate::util::{cmd::AskError, timestamp::NsTimestamp, uuid::UuidString};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

// TODO
#[derive(Deserialize, Serialize)]
pub struct PerformanceDatabase {
    pub format_version: i32,
    pub performances: Vec<Box<dyn Performance>>,
}

pub type PerformanceMetadata = HashMap<String, String>;

#[typetag::serde(tag = "game")]
pub trait Performance: Debug {
    fn score(&self) -> f64;
    fn proof(&self) -> Vec<UuidString>;
    fn timestamp(&self) -> NsTimestamp;
    fn comment(&self) -> Option<String>;
    fn metadata(&self) -> PerformanceMetadata;
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        unimplemented!()
    }
}
