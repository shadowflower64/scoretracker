use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, FromSql, ToSql, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[postgres(name = "proof_visibility", rename_all = "snake_case")]
pub enum ProofVisibility {
    #[default]
    /// This proof is visible to the owner only - details and file contents can only be fetched by the proof owner.
    Private,

    /// This proof is visible to everybody - details and file contents can be fetched by anyone who asks for it.
    Public,
}
