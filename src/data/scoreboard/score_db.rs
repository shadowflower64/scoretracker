use crate::data::scoreboard::{r#match::MatchTrait, performance::PerformanceTrait};
use serde::{Deserialize, Serialize};

// TODO
#[derive(Deserialize, Serialize)]
pub struct ScoreDatabase {
    pub format_version: i32,
    pub matches: Vec<Box<dyn MatchTrait>>,
    pub performances: Vec<Box<dyn PerformanceTrait>>,
}
