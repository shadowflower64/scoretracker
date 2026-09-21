use crate::toolkit::cmd::CmdError;
use function_name::named;
use scoretracker::data::game::game_instance_from_id;
use scoretracker::db::Database;
use scoretracker::success_npr;
use scoretracker::{info_npr, log_fn_name, util::command_line::ask_yn};

#[named]
pub fn add_performance(game_id: String) -> Result<(), CmdError> {
    log_fn_name!("cmd" : auto);

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

#[named]
pub fn add_player(name: String) -> Result<(), CmdError> {
    log_fn_name!("cmd" : auto);
    info_npr!("adding new player: '{name}'");

    smol::block_on(async {
        let mut db = Database::connect_and_spawn_smol_with_toolkit().await?;
        let (player_uuid, is_new) = db.get_or_insert_player(&name).await?;

        if is_new {
            success_npr!("player created: {player_uuid}");
            println!("{}", player_uuid);
            Ok(())
        } else {
            println!("{}", player_uuid);
            Err(CmdError::PlayerAlreadyInDatabase(player_uuid))
        }
    })
}
