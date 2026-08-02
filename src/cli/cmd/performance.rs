use crate::cmd::CmdError;
use scoretracker::data::game::game_instance_from_id;
use scoretracker::{info_npr, log_fn_name, util::command_line::ask_yn};

pub fn add(game_id: String) -> Result<(), CmdError> {
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
