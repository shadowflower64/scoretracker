use std::ops::{Deref, DerefMut};

use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod library;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String)]
pub struct RelativePathBufString(pub RelativePathBuf);

impl Deref for RelativePathBufString {
    type Target = RelativePathBuf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativePathBufString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
