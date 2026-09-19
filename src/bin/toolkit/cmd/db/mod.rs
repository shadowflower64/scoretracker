use crate::toolkit::error::CmdError;
use function_name::named;
use postgres::{Client, NoTls};
use scoretracker::config::secrets::SecretsConfig;
use scoretracker::log_fn_name;

pub const INIT_DB_SCRIPT: &str = include_str!("init_db.sql");

fn connect_to_db(secrets: &SecretsConfig) -> Result<Client, postgres::Error> {
    Client::connect(secrets.database_connection_params.as_ref().map(String::as_str).unwrap_or(""), NoTls)
}

#[named]
pub fn init(database_name: String) -> Result<(), CmdError> {
    log_fn_name!(auto);

    let secrets = SecretsConfig::get().map_err(CmdError::SecretsConfigError)?;
    let mut client = connect_to_db(secrets)?;
    client.execute(INIT_DB_SCRIPT, &[&database_name])?;

    Ok(())
}
