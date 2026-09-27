use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use crate::sql_json_impl;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
// FIXME: this is an IndexMap but when it gets written to the database it gets saved as jsonb
// which does not guarantee the preservation of insertion order (i think??).
// idk what to do about this yet
// it would be nice to preserve the insertion order/allow for nice top-level ordering of values here, but i guess it is not necessary.
pub struct ArbitraryMetadata(pub IndexMap<String, serde_json::Value>);

impl ArbitraryMetadata {
    pub fn new() -> Self {
        Self(IndexMap::new())
    }
}

sql_json_impl! {ArbitraryMetadata}

impl Deref for ArbitraryMetadata {
    type Target = IndexMap<String, serde_json::Value>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ArbitraryMetadata {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// impl JsonSchema for ArbitraryMetadata {
//     fn schema_name() -> Cow<'static, str> {
//         "ArbitraryMetadata".into()
//     }
//     fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
//         json_schema!({
//             "type": "object"
//         })
//     }
// }
