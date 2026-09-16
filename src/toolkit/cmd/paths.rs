use crate::error::CmdError;
use crate::server::config::ServerConfig;
use function_name::named;
use scoretracker::config::secrets::SecretsConfig;
use scoretracker::config::toolkit::ToolkitConfig;
use scoretracker::config::{library_tab::LibraryTab, toml::TomlConfig};
use scoretracker::log_fn_name;
use scoretracker::util::dirs::{config_dir, log_dir, project_temp_dir};
use std::path::Path;

#[named]
pub fn show() -> Result<(), CmdError> {
    log_fn_name!(auto);

    fn print_path(header: &str, value: &Path) {
        println!("{header:>30} = {value:?}");
    }

    print_path("config dir", &config_dir());
    print_path("log dir", &log_dir());
    print_path("project temp dir", &project_temp_dir());

    print_path("toolkit config", &ToolkitConfig::default_path());
    print_path("server config", &ServerConfig::default_path());
    print_path("shared secrets config", &SecretsConfig::default_path());
    print_path("library tab", &LibraryTab::default_path());

    Ok(())
}
