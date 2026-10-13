use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api::RelativePathBufString,
    data::library::{sha256::Sha256Hash, stpl_url::LibraryDomain},
    util::uuid::UuidString,
};

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ScanFullRequest {
    pub library_domain: LibraryDomain,
    pub authenticity_string: String,
    pub sha256_hashes: HashMap<RelativePathBufString, Sha256Hash>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ScanFullResponse {
    pub proof_uuids: HashMap<RelativePathBufString, UuidString>,
}
