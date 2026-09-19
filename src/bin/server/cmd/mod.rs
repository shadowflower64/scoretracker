use crate::server::error::ServerError;
use scoretracker::cli::cmdline_context::CmdlineContext;

pub fn handle_command(context: &mut CmdlineContext) -> Result<(), ServerError> {
    let mut ctx = context.with_error_type::<ServerError>();
    match ctx.cmd()? {
        "init" => {
            use crate::server::{config::ServerConfig, error::ServerError};
            use scoretracker::{config::toml::TomlConfig, success_npr};

            let path = ServerConfig::default_path();
            ServerConfig::default().write_new(&path).map_err(ServerError::ServerConfigError)?;
            success_npr!("config successfully written to: {path:?}");
            Ok(())
        }
        "start" => {
            use crate::server::http::start::http_server_start;
            http_server_start()?;
            Ok(())
        }
        _ => ctx.unknown_cmd(),
    }
}
