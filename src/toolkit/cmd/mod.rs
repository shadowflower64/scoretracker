use crate::arg::{CmdlineArgument, parse_arg, parse_arg_opt};
use crate::cmd;
use crate::cmd::CmdError::NoCommandProvided;
use crate::error::CmdError;
use crate::server::config::ServerConfig;
use crate::server::start::server_main;
use scoretracker::config::Config;
use scoretracker::config::libraries::LibraryTable;
use scoretracker::data::library::stpl_url::LibraryDomain;
use scoretracker::hive::jobs::cut_library_video::CutLibraryVideoJob;
use scoretracker::hive::jobs::process_library_video::{Operation, ProcessLibraryVideoJob};
use scoretracker::util::lossless_cut_project::LlcProj;
use scoretracker::util::timestamp::NsLocalTimestamp;
use scoretracker::{info_npr, success_npr};
use std::path::PathBuf;

pub mod automark;
pub mod config;
pub mod hive;
pub mod library;
pub mod log;
pub mod schema;
pub mod scoreboard;
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

    pub fn cmd_opt(&mut self) -> Result<Option<&str>, CmdError> {
        let cmd_opt = self.arguments.get(self.top).map(String::as_str);
        if let Some(cmd) = cmd_opt {
            if self.full_command_name.is_empty() {
                self.full_command_name = cmd.to_string();
            } else {
                self.full_command_name = format!("{}:{cmd}", self.full_command_name);
            }
            self.top += 1;
        }
        Ok(cmd_opt)
    }

    fn last_cmd(&mut self) -> Option<&str> {
        self.arguments.get(self.top - 1).map(String::as_str)
    }

    pub fn pull_arg<T: CmdlineArgument>(&mut self, name: &str, description: &str) -> Result<T, CmdError> {
        let arg = parse_arg(
            self.arguments.get(self.top).map(String::as_str),
            name,
            description,
            &self.full_command_name,
        )?;
        self.top += 1;
        Ok(arg)
    }

    pub fn pull_arg_opt<T: CmdlineArgument>(&mut self, name: &str, description: &str) -> Result<Option<T>, CmdError> {
        let arg = parse_arg_opt(
            self.arguments.get(self.top).map(String::as_str),
            name,
            description,
            &self.full_command_name,
        )?;
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
    let mut ctx = CmdlineContext::new(arguments);

    match ctx.cmd()? {
        "hello" => {
            info_npr!("hello world!");
            Ok(())
        }
        "automark" => {
            let library_dir: Option<PathBuf> = ctx.pull_arg_opt("library_dir", "path of the library directory")?;
            let library_dir = library_dir
                .map(Ok)
                .unwrap_or_else(|| Config::load().map(|x| x.default_library_dir_path.clone()))
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
                        cmd::hive::add_task(CutLibraryVideoJob {
                            source_path,
                            source_proof_uuid_precondition_check: None,
                            cut_start_point: cut_start_point.map(NsLocalTimestamp::from_secs_f64),
                            cut_end_point: cut_end_point.map(NsLocalTimestamp::from_secs_f64),
                            destination_path,
                        })
                    }
                    "process-video" => {
                        let source_path: PathBuf = ctx.pull_arg("source_path", "source path to dry video")?;
                        let destination_path: PathBuf = ctx.pull_arg("destination_path", "destination path to wet video")?;
                        let processing_type: Operation =
                            ctx.pull_arg("processing_type", "type/quality preset of video compression to do")?;
                        cmd::hive::add_task(ProcessLibraryVideoJob {
                            source_path,
                            source_proof_uuid_precondition_check: None,
                            operation: processing_type,
                            destination_path,
                        })
                    }
                    "execute-llc" => {
                        let source_path: PathBuf = ctx.pull_arg("source_path", "source path to cloth video")?;
                        let file_stem = source_path.file_stem().expect("todo: invalid file name").to_string_lossy();
                        let llc_proj_file_name = format!("{file_stem}-proj.llc");
                        let llc = LlcProj::load_from_file(llc_proj_file_name).expect("todo: invalid llc proj");
                        for (i, segment) in llc.cut_segments.iter().enumerate() {
                            let segment_number = if llc.cut_segments.len() == 1 { None } else { Some(i + 1) };
                            let segment_fragment = segment_number.map(|num| format!("-seg{num}")).unwrap_or_default();
                            let cut_start_point = NsLocalTimestamp::from_secs_f64(segment.start);
                            let cut_end_point = NsLocalTimestamp::from_secs_f64(segment.end);

                            //{file_stem}-00.07.40.660-00.09.48.941-stcut.mkv
                            //{file_stem}-00.07.40.660-00.09.48.941-seg4-stcut.mkv
                            let file_name = format!(
                                "{file_stem}-{}-{}{segment_fragment}-stcut.mkv",
                                cut_start_point.to_string_within_filename(),
                                cut_end_point.to_string_within_filename()
                            );
                            let destination_path: PathBuf = source_path.with_file_name(file_name);

                            cmd::hive::add_task(CutLibraryVideoJob {
                                source_path: source_path.to_path_buf(),
                                source_proof_uuid_precondition_check: None,
                                cut_start_point: Some(cut_start_point),
                                cut_end_point: Some(cut_end_point),
                                destination_path,
                            })?;
                        }
                        Ok(())
                    }
                    "fold-video" => {
                        let source_path: PathBuf = ctx.pull_arg("source_path", "source path to dry video")?;
                        let file_name = format!(
                            "{}-stfolded.mkv",
                            source_path.file_stem().expect("todo: invalid file name").to_string_lossy()
                        );
                        let destination_path: PathBuf = source_path.with_file_name(file_name);
                        cmd::hive::add_task(ProcessLibraryVideoJob {
                            source_path,
                            source_proof_uuid_precondition_check: None,
                            operation: Operation::CompressFoldVideo,
                            destination_path,
                        })
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
                let library_dir: PathBuf = if let Some(arg) = ctx.pull_arg_opt("library_dir", "path of the library directory")? {
                    arg
                } else {
                    Config::load()
                        .map_err(CmdError::ConfigReadError)?
                        .default_library_dir_path
                        .to_owned()
                };
                cmd::library::rescan(&library_dir)
            }
            "remove-domain" => {
                let library_domain: LibraryDomain = ctx.pull_arg("library_domain", "library domain name")?;
                cmd::library::remove_domain(library_domain)
            }
            "table" => match ctx.cmd()? {
                "init" => {
                    let path = LibraryTable::default_path();
                    LibraryTable::default().write_new(&path)?;
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
        "scoreboard" => match ctx.cmd()? {
            "init" => cmd::scoreboard::init(),
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
            "clean" => cmd::schema::clean(),
            _ => ctx.unknown_cmd(),
        },
        "server" => match ctx.cmd()? {
            "init" => {
                let path = ServerConfig::default_path();
                ServerConfig::default().write_new(&path)?;
                success_npr!("config successfully written to: {path:?}");
                Ok(())
            }
            "start" => {
                // info_npr!("starting web application");
                server_main()?;
                Ok(())
            }
            _ => ctx.unknown_cmd(),
        },
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
        "vitals" => match ctx.cmd_opt()? {
            Some("all") | None => cmd::vitals::check_all(),
            _ => ctx.unknown_cmd(),
        },
        _ => ctx.unknown_cmd(),
    }
}
