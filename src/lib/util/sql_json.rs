#[macro_export]
/// Provides implementations for FromSql and ToSql by (de-)serializing the value using serde_json.
/// Good for fields that have the `json` or `jsonb` column type.
macro_rules! sql_json_impl {
    ($sql_json_type:ty) => {
        impl<'a> postgres_types::FromSql<'a> for $sql_json_type {
            fn accepts(ty: &postgres_types::Type) -> bool {
                <serde_json::Value as postgres_types::FromSql>::accepts(ty)
            }
            fn from_sql(ty: &postgres_types::Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let value = serde_json::Value::from_sql(ty, raw)?;
                let details = serde_json::from_value(value)?;
                Ok(details)
            }
        }

        impl postgres_types::ToSql for $sql_json_type {
            fn accepts(ty: &postgres_types::Type) -> bool
            where
                Self: Sized,
            {
                <serde_json::Value as postgres_types::ToSql>::accepts(ty)
            }
            fn to_sql(
                &self,
                ty: &postgres_types::Type,
                out: &mut actix_web::web::BytesMut,
            ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
            where
                Self: Sized,
            {
                let value = serde_json::to_value(self)?;
                value.to_sql(ty, out)
            }
            postgres_types::to_sql_checked! {}
        }
    };
}
