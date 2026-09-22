use crate::toolkit::cmd::CmdError;
use function_name::named;
use scoretracker::db::Database;
use scoretracker::success_npr;
use scoretracker::{info_npr, log_fn_name};

#[named]
pub fn add(name: String) -> Result<(), CmdError> {
    log_fn_name!("cmd" : auto);
    info_npr!("adding new player: '{name}'");

    smol::block_on(async {
        let mut db = Database::connect_with_tokio_for_toolkit().await?;
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
