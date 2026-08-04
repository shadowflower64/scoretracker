use crate::arg::{CmdlineArgument, parse_arg, parse_arg_opt};
use crate::cmd::CmdError::NoCommandProvided;
use crate::cmd::{self};
use crate::error::CmdError;
use scoretracker::config::Config;
use scoretracker::data::library::stpl_url::{LibraryDomain, LibraryDomainName};
use scoretracker::hive::job::Job;
use scoretracker::info_npr;
use scoretracker::util::timestamp::NsTimestamp;
use std::path::PathBuf;

pub mod automark;
pub mod config;
pub mod hive;
pub mod library;
pub mod performance;
pub mod player;
pub mod spreadsheet;
pub mod vitals;

pub struct CmdlineContext<'a> {
    arguments: &'a [String],
    full_command_name: String,
    top: usize,
}

impl<'a> CmdlineContext<'a> {
    pub fn cmd(&mut self) -> Result<&str, CmdError> {
        let cmd = self.arguments.get(self.top).map(String::as_str).ok_or_else(|| {
            if self.full_command_name.is_empty() {
                NoCommandProvided
            } else {
                CmdError::NoSubcommandProvided {
                    cmd: self.full_command_name.clone(),
                }
            }
        })?;
        if self.full_command_name.is_empty() {
            self.full_command_name = cmd.to_string();
        } else {
            self.full_command_name = format!("{}:{cmd}", self.full_command_name);
        }
        self.top += 1;
        Ok(cmd)
    }

    fn last_cmd(&mut self) -> Option<&String> {
        self.arguments.get(self.top - 1)
    }

    pub fn pull_arg<T: CmdlineArgument>(&mut self, name: &str, description: &str) -> Result<T, CmdError> {
        let arg = parse_arg(self.arguments.get(self.top), name, description, &self.full_command_name)?;
        self.top += 1;
        Ok(arg)
    }

    pub fn pull_arg_opt<T: CmdlineArgument>(&mut self, name: &str, description: &str) -> Result<Option<T>, CmdError> {
        let arg = parse_arg_opt(self.arguments.get(self.top), name, description, &self.full_command_name)?;
        self.top += 1;
        Ok(arg)
    }

    pub fn unknown_cmd(&mut self) -> Result<(), CmdError> {
        let matched = self
            .last_cmd()
            .expect("invalid use of CmdlineContext::unknown_cmd - use it in a match, after calling CmdlineContext::cmd")
            .to_owned();
        if self.full_command_name.is_empty() {
            Err(CmdError::UnknownCommand { cmd: matched })
        } else {
            Err(CmdError::UnknownSubcommand {
                cmd: self.full_command_name.clone(),
                subcmd: matched,
            })
        }
    }

    pub fn new(arguments: &'a [String]) -> Self {
        Self {
            arguments,
            full_command_name: String::new(),
            top: 1,
        }
    }
}

#[allow(unused_assignments)]
pub fn handle_command(arguments: &[String]) -> Result<(), CmdError> {
    type E = CmdError;
    let mut ctx = CmdlineContext::new(arguments);

    // fcn - full command name
    match ctx.cmd()? {
        "hello" => {
            info_npr!("hello world!");
            Ok(())
        }
        "spreadsheet" => match ctx.cmd()? {
            "import-org-ods" => {
                let path: PathBuf = ctx.pull_arg("path", "path of the ods spreadsheet file")?;
                cmd::spreadsheet::import_org_ods(&path)
            }
            "import-org-xlsx" => {
                let path: PathBuf = ctx.pull_arg("path", "path of the xlsx spreadsheet file")?;
                cmd::spreadsheet::import_org_xlsx(&path)
            }
            _ => ctx.unknown_cmd(),
        },
        "config" => match ctx.cmd()? {
            "init" => cmd::config::init(),
            "show" => cmd::config::show(),
            "set" => {
                let config_key: String = ctx.pull_arg("config_key", "name of the key to change in the configuration")?;
                let config_value: String = ctx.pull_arg("config_value", "new value for the selected key")?;
                cmd::config::set(config_key, config_value)
            }
            _ => ctx.unknown_cmd(),
        },
        "library" => match ctx.cmd()? {
            "init" => {
                let library_dir: PathBuf = ctx.pull_arg("library_dir", "path of the library directory")?;
                let library_domain_name: LibraryDomainName = ctx.pull_arg("library_domain_name", "library domain name")?;
                cmd::library::init(&library_dir, library_domain_name)
            }
            "rescan" => {
                let library_dir: PathBuf = ctx.pull_arg("library_dir", "path of the library directory")?;
                cmd::library::rescan(&library_dir).map_err(E::LibraryScanError)
            }
            "rescan-default" => {
                let config = Config::load().map_err(CmdError::ConfigReadError)?;
                cmd::library::rescan(&config.default_library_dir_path).map_err(E::LibraryScanError)
            }
            "remove-domain" => {
                let library_domain: LibraryDomain = ctx.pull_arg("library_domain", "library domain name")?;
                cmd::library::remove_domain(library_domain).map_err(E::DatabaseError)
            }
            _ => ctx.unknown_cmd(),
        },
        "hive" => match ctx.cmd()? {
            "worker" => match ctx.cmd()? {
                "spawn" => {
                    let persistent: bool = ctx.pull_arg("persistent", "should the worker stay alive after finishing a task?")?;
                    cmd::hive::spawn_worker(persistent)
                }
                _ => ctx.unknown_cmd(),
            },
            "task" => match ctx.cmd()? {
                "add" => match ctx.cmd()? {
                    "cut-video" => {
                        let source_path: PathBuf = ctx.pull_arg("source_path", "source path to cloth video")?;
                        let destination_path: PathBuf = ctx.pull_arg("destination_path", "destination path to fragment video")?;
                        let cut_start_point: Option<f64> = ctx.pull_arg_opt("cut_start_point", "timestamp to start of cut (in seconds)")?;
                        let cut_end_point: Option<f64> = ctx.pull_arg_opt("cut_end_point", "timestamp to end of cut (in seconds)")?;
                        cmd::hive::add_task(Job::CutLibraryVideo {
                            source_path,
                            cut_start_point: cut_start_point.map(NsTimestamp::from_secs_f64),
                            cut_end_point: cut_end_point.map(NsTimestamp::from_secs_f64),
                            destination_path,
                        })
                    }
                    _ => ctx.unknown_cmd(),
                },
                _ => ctx.unknown_cmd(),
            },
            _ => ctx.unknown_cmd(),
        },
        "vitals" => match ctx.cmd()? {
            "all" => cmd::vitals::check_all(),
            _ => ctx.unknown_cmd(),
        },
        "performance" => match ctx.cmd()? {
            "add" => {
                let game_id: String = ctx.pull_arg("game_id", "id of the game to add a performance for")?;
                cmd::performance::add(game_id)
            }
            _ => ctx.unknown_cmd(),
        },
        "player" => match ctx.cmd()? {
            "add" => {
                let name: String = ctx.pull_arg("name", "name of the player")?;
                cmd::player::add(name)
            }
            _ => ctx.unknown_cmd(),
        },
        _ => ctx.unknown_cmd(),
    }
}
