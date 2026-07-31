//! Data structures for DJMAX RESPECT V.

use crate::data::game::Game;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DJMAXRESPECTV;

#[typetag::serde(name = "djmax_respect_v")]
impl Game for DJMAXRESPECTV {
    fn pretty_name(&self) -> &'static str {
        "DJMAX RESPECT V"
    }
    fn url_shortname(&self) -> &'static str {
        "djmax_respect_v"
    }
}
