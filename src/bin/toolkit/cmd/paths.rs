use crate::server::config::ServerConfig;
use crate::toolkit::error::CmdError;
use function_name::named;
use scoretracker::config::toolkit::ToolkitConfig;
use scoretracker::config::{library_tab::LibraryTab, toml::TomlConfig};
use scoretracker::log_fn_name;
use scoretracker::util::dirs::{config_dir, log_dir, project_temp_dir};
use std::path::Path;

#[named]
pub fn show() -> Result<(), CmdError> {
    log_fn_name!(auto);

    fn pretty_print_path(header: &str, value: &Path) {
        println!("{header:>30} = {value:?}");
    }

    pretty_print_path("config_dir", &config_dir());
    pretty_print_path("log_dir", &log_dir());
    pretty_print_path("project_temp_dir", &project_temp_dir());

    pretty_print_path("toolkit_config", &ToolkitConfig::default_path());
    pretty_print_path("server_config", &ServerConfig::default_path());
    pretty_print_path("library_tab", &LibraryTab::default_path());

    Ok(())
}
