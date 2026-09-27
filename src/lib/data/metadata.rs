use indexmap::IndexMap;
use postgres_types::FromSql;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

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

impl<'a> FromSql<'a> for ArbitraryMetadata {
    fn from_sql(ty: &postgres_types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = serde_json::Value::from_sql(ty, raw)?;
        let index_map = serde_json::from_value(value)?;
        Ok(Self(index_map))
    }
    fn accepts(ty: &postgres_types::Type) -> bool {
        serde_json::Value::accepts(ty)
    }
}

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
