use crate::cmd::CmdError;
use scoretracker::config::Config;
use scoretracker::data::scoreboard::player::PlayerDatabase;
use scoretracker::util::filelocked::FileLockableData;
use scoretracker::warn_npr;
use scoretracker::{info_npr, log_fn_name};

pub fn add(name: String) -> Result<(), CmdError> {
    log_fn_name!("cmd:player_add");
    info_npr!("adding new player: '{name}'");

    let player_db_path = Config::load().unwrap().player_database_path(); // TODO: error handling
    let mut player_db = PlayerDatabase::lock_and_read(player_db_path, None).unwrap(); // TODO: error handling
    let result = player_db.add(&name);
    player_db.save_and_close().unwrap(); // TODO: error handling

    if let Ok(uuid) = result {
        info_npr!("player created: {uuid}");
        println!("{}", uuid);
    } else if let Err(uuid) = result {
        warn_npr!("player was already in database: {uuid}");
        println!("{}", uuid);
        // TODO: exitcode shouldn't be 0
    } else {
        unreachable!()
    }
    Ok(())
}
