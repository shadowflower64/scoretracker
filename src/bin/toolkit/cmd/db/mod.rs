pub mod performance;
pub mod player;

use std::{borrow::Cow, fs, path::Path};

use crate::toolkit::error::CmdError;
use chrono::Local;
use function_name::named;
use scoretracker::{
    config::toolkit::ToolkitConfig,
    db::{Database, DbError, schema_name::SafeSchemaName},
    log_fn_name, success,
};

pub const INIT_DB_SCRIPT: &str = include_str!("init_db.sql");

#[named]
pub fn init(schema_name: SafeSchemaName) -> Result<(), CmdError> {
    log_fn_name!(auto);

    smol::block_on(async {
        let config = ToolkitConfig::global().map_err(CmdError::ToolkitConfigError)?;
        let db = Database::connect_with_tokio(
            &config
                .database_connection
                .as_ref()
                .ok_or(DbError::ToolkitConfigNoDbConnectionString)?,
            &schema_name,
        )
        .await?;

        db.client.batch_execute(INIT_DB_SCRIPT).await?;
        success!("successfully executed `init_db.sql` script");

        Ok(())
    })
}

pub fn current_time_condensed_string() -> String {
    let current_time = Local::now();
    current_time.format("%Y%m%d%H%M%S").to_string()
}

#[named]
pub fn export_jsonl(export_dir: &Path) -> Result<(), CmdError> {
    log_fn_name!(auto);

    let export_dir = if export_dir.exists() {
        Cow::Owned(export_dir.join(format!("export_{}", current_time_condensed_string())))
    } else {
        Cow::Borrowed(export_dir)
    };
    fs::create_dir_all(&export_dir)?;

    smol::block_on(async {
        let mut db = Database::connect_with_tokio_for_toolkit().await?;
        let export = db.export_all().await?;
        serde_jsonlines::write_json_lines(export_dir.join("players.jsonl"), export.players.iter())?;
        serde_jsonlines::write_json_lines(export_dir.join("proofs.jsonl"), export.proofs.iter())?;
        serde_jsonlines::write_json_lines(export_dir.join("performances.jsonl"), export.performances.iter())?;
        serde_jsonlines::write_json_lines(export_dir.join("matches.jsonl"), export.matches.iter())?;

        success!("exported database to: {export_dir:?}");

        Ok(())
    })
}
