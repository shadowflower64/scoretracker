use crate::cmd::CmdError;
use scoretracker::config::Config;
use scoretracker::data::scoreboard::player::PlayerDatabase;
use scoretracker::util::filelocked::FileLockableData;
use scoretracker::{info_npr, log_fn_name, success_npr};

pub fn add(name: String) -> Result<(), CmdError> {
    log_fn_name!("cmd:player_add");
    info_npr!("adding new player: '{name}'");

    let player_db_path = Config::load().map_err(CmdError::ConfigReadError)?.player_database_path();
    let mut player_db = PlayerDatabase::lock_and_read(player_db_path, None).map_err(CmdError::PlayerDatabaseOpenError)?;
    let result = player_db.add(&name);
    player_db.save_and_close().map_err(CmdError::PlayerDatabaseWriteError)?;

    match result {
        Ok(uuid) => {
            success_npr!("player created: {uuid}");
            println!("{}", uuid);
            Ok(())
        }
        Err(uuid) => {
            println!("{}", uuid);
            Err(CmdError::PlayerAlreadyInDatabase(uuid))
        }
    }
}
