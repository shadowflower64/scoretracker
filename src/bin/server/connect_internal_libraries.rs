use function_name::named;
use scoretracker::config::{library_tab::LibraryTab, toml::TomlConfig};

use crate::server::library_hall::LibraryConnections;

#[named]
pub fn connect_internal_libraries() -> LibraryConnections {
    let tab = LibraryTab::load().expect("todo: error handling");
    let mut connections = LibraryConnections::new();
    connections.add_internal_mirrors(tab.scan());
    connections
}
