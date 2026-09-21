use crate::toolkit::cmd;
use crate::toolkit::cmd::library::LibraryIdentifier;
use crate::toolkit::error::CmdError;
use scoretracker::cli::cmdline_context::CmdlineContext;
use scoretracker::cli::cmdline_error::CmdlineError;
use scoretracker::config::LegacyConfig;
use scoretracker::config::library_tab::LibraryTab;
use scoretracker::config::toml::TomlConfig;
use scoretracker::data::library::stpl_url::{LibraryDomain, StplUrl};
use scoretracker::hive::jobs::cut_library_video::CutLibraryVideoJob;
use scoretracker::hive::jobs::process_library_video::{Operation, ProcessLibraryVideoJob};
use scoretracker::util::timestamp::NsLocalTimestamp;
use scoretracker::{info_npr, success_npr};
use std::path::PathBuf;

pub mod automark;
pub mod config;
pub mod db;
pub mod hive;
pub mod library;
pub mod log;
pub mod paths;
pub mod schema;
pub mod scoreboard;
pub mod spreadsheet;
pub mod version;
pub mod vitals;

pub fn handle_command(context: &mut CmdlineContext) -> Result<(), CmdError> {
    let mut ctx = context.with_error_type::<CmdError>();
    match ctx.cmd()? {
        "hello" => {
            info_npr!("hello world!");
            Ok(())
        }
        "automark" => {
            let library_dir: Option<PathBuf> = ctx.pull_arg_opt("library_dir", "path of the library directory")?;
            let library_dir = library_dir
                .map(Ok)
                .unwrap_or_else(|| LegacyConfig::load().map(|x| x.default_library_dir_path.clone()))
                .map_err(CmdError::ConfigReadError)?;
            cmd::automark::automark_library_files(library_dir)
        }
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
        "db" => match ctx.cmd()? {
            "init" => {
                let schema_name: String = ctx.pull_arg("schema_name", "name for the new schema to create in the database")?;
                cmd::db::init(schema_name)
            }
            _ => ctx.unknown_cmd(),
        },
        "hive" => match ctx.cmd()? {
            "worker" => match ctx.cmd()? {
                // TODO: rename to "start", "spawn" implies spawning a background process
                "spawn" => {
                    // let persistent: bool = ctx.pull_arg("persistent", "should the worker stay alive after finishing a task?")?;
                    cmd::hive::start_worker(/*persistent*/)
                }
                _ => ctx.unknown_cmd(),
            },
            "task" => match ctx.cmd()? {
                "add" => match ctx.cmd()? {
                    "cut-video" => {
                        let source: StplUrl = ctx.pull_arg("source", "source url to cloth video")?;
                        let destination: StplUrl = ctx.pull_arg("destination", "destination url to fragment video")?;
                        let cut_start_point: Option<f64> = ctx.pull_arg_opt("cut_start_point", "timestamp to start of cut (in seconds)")?;
                        let cut_end_point: Option<f64> = ctx.pull_arg_opt("cut_end_point", "timestamp to end of cut (in seconds)")?;
                        cmd::hive::add_task(CutLibraryVideoJob {
                            source,
                            source_proof_uuid_precondition_check: None,
                            destination,
                            cut_start_point: cut_start_point.map(NsLocalTimestamp::from_secs_f64),
                            cut_end_point: cut_end_point.map(NsLocalTimestamp::from_secs_f64),
                        })
                    }
                    "process-video" => {
                        let source: StplUrl = ctx.pull_arg("source", "source url to dry video")?;
                        let destination: StplUrl = ctx.pull_arg("destination", "destination url to wet video")?;
                        let operation: Operation = ctx.pull_arg("operation", "type/quality preset of video compression to do")?;
                        cmd::hive::add_task(ProcessLibraryVideoJob {
                            source,
                            source_proof_uuid_precondition_check: None,
                            destination,
                            operation,
                        })
                    }
                    "execute-llc" => {
                        let source_paths: Vec<PathBuf> = ctx.pull_args("source_path", "source path to cloth video")?;
                        for source_path in source_paths {
                            cmd::hive::add_task_execute_llc(source_path)?
                        }
                        Ok(())
                    }
                    "fold-video" => {
                        let source_paths: Vec<PathBuf> = ctx.pull_args("source_path", "source path to dry video")?;
                        for source_path in source_paths {
                            cmd::hive::add_task_fold_video(source_path)?
                        }
                        Ok(())
                    }
                    "mess-up-video" => {
                        let source_paths: Vec<PathBuf> = ctx.pull_args("source_path", "source path to dry video")?;
                        for source_path in source_paths {
                            cmd::hive::add_task_mess_up_video(source_path)?
                        }
                        Ok(())
                    }
                    "crumple-video" => {
                        let source_paths: Vec<PathBuf> = ctx.pull_args("source_path", "source path to dry video")?;
                        for source_path in source_paths {
                            cmd::hive::add_task_crumple_video(source_path)?
                        }
                        Ok(())
                    }
                    "shred-video" => {
                        let source_paths: Vec<PathBuf> = ctx.pull_args("source_path", "source path to dry video")?;
                        for source_path in source_paths {
                            cmd::hive::add_task_shred_video(source_path)?
                        }
                        Ok(())
                    }
                    _ => ctx.unknown_cmd(),
                },
                _ => ctx.unknown_cmd(),
            },
            _ => ctx.unknown_cmd(),
        },
        "library" => match ctx.cmd()? {
            "init" => {
                let library_dir: PathBuf = ctx.pull_arg("library_dir", "path of the library directory")?;
                let library_domain: LibraryDomain = ctx.pull_arg("library_domain", "library domain name")?;
                cmd::library::init(&library_dir, library_domain)
            }
            "install" => {
                let library_dir: PathBuf = ctx.pull_arg("library_dir", "path of the library directory")?;
                cmd::library::install(&library_dir)
            }
            "rescan" => {
                let library: LibraryIdentifier =
                    if let Some(arg) = ctx.pull_arg_opt("library", "library domain name or path to the library directory")? {
                        Ok(arg)
                    } else if let Some(default) = &LegacyConfig::load().map_err(CmdError::ConfigReadError)?.default_library {
                        Ok(LibraryIdentifier::DomainName(default.clone()))
                    } else {
                        Err(CmdlineError::ArgumentNotProvided {
                            cmd: "library:rescan".to_string(),
                            arg_name: "library".to_string(),
                            arg_desc: "library domain name or path to the library directory".to_string(),
                        })
                    }?;
                let dir_path = library.dir_path().expect("todo: invalid library domain/path");
                cmd::library::rescan(&dir_path)
            }
            "remove-domain" => {
                let library_domain: LibraryDomain = ctx.pull_arg("library_domain", "library domain name")?;
                cmd::library::remove_domain(library_domain)
            }
            "table" => match ctx.cmd()? {
                "init" => {
                    let path = LibraryTab::default_path();
                    LibraryTab::default().write_new(&path).map_err(CmdError::LibraryTableError)?;
                    success_npr!("empty library table successfully written to: {path:?}");
                    Ok(())
                }
                _ => ctx.unknown_cmd(),
            },
            _ => ctx.unknown_cmd(),
        },
        "log" => match ctx.cmd()? {
            "open" => cmd::log::open(),
            _ => ctx.unknown_cmd(),
        },
        "logs" => cmd::log::open(),
        "paths" => match ctx.cmd_opt()? {
            None | Some("show") => cmd::paths::show(),
            _ => ctx.unknown_cmd(),
        },
        "scoreboard" => match ctx.cmd()? {
            "performance" => match ctx.cmd()? {
                "add" => {
                    let game_id: String = ctx.pull_arg("game_id", "id of the game to add a performance for")?;
                    cmd::scoreboard::add_performance(game_id)
                }
                _ => ctx.unknown_cmd(),
            },
            "player" => match ctx.cmd()? {
                "add" => {
                    let name: String = ctx.pull_arg("name", "name of the player")?;
                    cmd::scoreboard::add_player(name)
                }
                _ => ctx.unknown_cmd(),
            },
            _ => ctx.unknown_cmd(),
        },
        "schema" => match ctx.cmd()? {
            "gen" => cmd::schema::gen_full(),
            "gen-json" => cmd::schema::gen_json(),
            "gen-types" => cmd::schema::gen_types(),

            #[cfg(feature = "include-server-in-toolkit")]
            "gen-api" => cmd::schema::gen_api(),

            "clean" => cmd::schema::clean(),
            _ => ctx.unknown_cmd(),
        },
        "server" => {
            if cfg!(feature = "include-server-in-toolkit") {
                Ok(crate::server::cmd::handle_command(context)?)
            } else {
                Err(CmdError::ServerNotIncluded)
            }
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
        "version" => cmd::version::version(),
        "vitals" => match ctx.cmd_opt()? {
            Some("all") | None => cmd::vitals::check_all(),
            _ => ctx.unknown_cmd(),
        },
        _ => ctx.unknown_cmd(),
    }
}
