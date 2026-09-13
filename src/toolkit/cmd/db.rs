use crate::error::CmdError;
use postgres::{Client, NoTls};

pub const INIT_DB_SCRIPT: &str = include_str!("init_db.sql");

fn connect_to_db() -> Result<Client, postgres::Error> {
    pub const DATABASE_CONNECTION_PARAMS: &str = "host=localhost user=postgres";
    Client::connect(DATABASE_CONNECTION_PARAMS, NoTls)
}

pub fn init(database_name: String) -> Result<(), CmdError> {
    let mut client = connect_to_db()?;

    client.execute(INIT_DB_SCRIPT, &[&database_name])?;

    Ok(())
}
