use std::{str::FromStr, sync::LazyLock};

use crate::toolkit::error::CmdError;
use function_name::named;
use postgres::{Client, NoTls};
use regex::Regex;
use scoretracker::{config::toolkit::ToolkitConfig, log_fn_name, success};

pub const INIT_DB_SCRIPT: &str = include_str!("init_db.sql");

#[named]
fn connect_to_db_sync(database_connection: &Option<String>) -> Result<Client, postgres::Error> {
    log_fn_name!(auto);

    let params = database_connection.as_ref().map(String::as_str).unwrap_or("");
    // info!("params: {params}");
    let config = postgres::Config::from_str(params)?;
    // info!("config: {config:?}");

    let client = config.connect(NoTls)?;
    success!("connected to database");
    Ok(client)
}

#[named]
pub fn init(schema_name: String) -> Result<(), CmdError> {
    log_fn_name!(auto);

    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z_]{1,64}$").expect("could not compile regex"));
    if !REGEX.is_match(&schema_name) {
        // TODO: pretty error handling
        panic!("invalid schema name: '{schema_name}' (needs to be use [a-z_] only)");
    }

    let config = ToolkitConfig::global().map_err(CmdError::SecretsConfigError)?;
    // info!("config: {config:?}");

    let mut client = connect_to_db_sync(&config.database_connection)?;

    let init_db_script_replaced = INIT_DB_SCRIPT.replace("$SCHEMA_NAME", &schema_name);
    client.batch_execute(&init_db_script_replaced)?;
    success!("successfully executed `init_db.sql` script");

    Ok(())
}
