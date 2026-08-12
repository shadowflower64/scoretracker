use crate::cmd::CmdError;
use crate::error::CmdError::{
    ConfigReadError, MatchDatabaseOpenError, MatchDatabaseWriteError, PerformanceDatabaseOpenError, PerformanceDatabaseWriteError,
    PlayerDatabaseOpenError, PlayerDatabaseWriteError,
};
use scoretracker::config::Config;
use scoretracker::data::game::game_instance_from_id;
use scoretracker::data::scoreboard::r#match::MatchDatabase;
use scoretracker::data::scoreboard::performance::PerformanceDatabase;
use scoretracker::data::scoreboard::player::PlayerDatabase;
use scoretracker::success_npr;
use scoretracker::util::filelocked::{FileLockableData, FileLockableDataDefault};
use scoretracker::{info_npr, log_fn_name, util::command_line::ask_yn};

pub fn init() -> Result<(), CmdError> {
    let config = Config::load().map_err(ConfigReadError)?;

    let player_database = PlayerDatabase::lock_and_read_or_default(config.player_database_path(), None).map_err(PlayerDatabaseOpenError)?;
    player_database.save_and_unlock().map_err(PlayerDatabaseWriteError)?;

    let match_database = MatchDatabase::lock_and_read_or_default(config.match_database_path(), None).map_err(MatchDatabaseOpenError)?;
    match_database.save_and_unlock().map_err(MatchDatabaseWriteError)?;

    let performance_database =
        PerformanceDatabase::lock_and_read_or_default(config.performance_database_path(), None).map_err(PerformanceDatabaseOpenError)?;
    performance_database.save_and_unlock().map_err(PerformanceDatabaseWriteError)?;

    Ok(())
}

pub fn add_performance(game_id: String) -> Result<(), CmdError> {
    log_fn_name!("cmd:performance_add");

    let game = game_instance_from_id(&game_id).ok_or(CmdError::NoGameWithId(game_id))?;
    info_npr!("adding new performance for {}", game.pretty_name());

    let mut performance = game.ask_for_performance_new()?;
    info_npr!("performance created:\n{:#?}", performance);

    while ask_yn("do you want to edit this performance?", None)? {
        performance.ask_for_performance_edit()?;
        println!("{:#?}", performance);
        info_npr!("performance updated:\n{:#?}", performance);
    }

    // TODO save performance to db
    // success_npr!("saved performance to database successfully");
    Ok(())
}

pub fn add_player(name: String) -> Result<(), CmdError> {
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
