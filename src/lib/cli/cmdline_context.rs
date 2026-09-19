use crate::cli::{
    cmdline_argument::{CmdlineArgument, parse_arg, parse_arg_opt},
    cmdline_error::CmdlineError,
};

pub struct CmdlineContext<'a> {
    arguments: &'a [String],
    full_command_name: String,
    top: usize,
}

impl<'a> CmdlineContext<'a> {
    pub fn cmd<E: From<CmdlineError>>(&mut self) -> Result<&str, E> {
        let cmd = self.arguments.get(self.top).map(String::as_str).ok_or_else(|| {
            if self.full_command_name.is_empty() {
                E::from(CmdlineError::NoCommandProvided)
            } else {
                E::from(CmdlineError::NoSubcommandProvided {
                    cmd: self.full_command_name.clone(),
                })
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

    pub fn cmd_opt<E: From<CmdlineError>>(&mut self) -> Result<Option<&str>, E> {
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

    pub fn peek(&mut self) -> Option<&str> {
        self.arguments.get(self.top).map(String::as_str)
    }

    pub fn pull_arg<T: CmdlineArgument, E: From<CmdlineError>>(&mut self, name: &str, description: &str) -> Result<T, E> {
        let arg = parse_arg(
            self.arguments.get(self.top).map(String::as_str),
            name,
            description,
            &self.full_command_name,
        )?;
        self.top += 1;
        Ok(arg)
    }

    pub fn pull_arg_opt<T: CmdlineArgument, E: From<CmdlineError>>(&mut self, name: &str, description: &str) -> Result<Option<T>, E> {
        let arg = parse_arg_opt(
            self.arguments.get(self.top).map(String::as_str),
            name,
            description,
            &self.full_command_name,
        )?;
        self.top += 1;
        Ok(arg)
    }

    pub fn pull_args<T: CmdlineArgument, E: From<CmdlineError>>(&mut self, name: &str, description: &str) -> Result<Vec<T>, E> {
        let mut vec = Vec::new();
        while self.peek().is_some() {
            let arg = parse_arg(
                self.arguments.get(self.top).map(String::as_str),
                name,
                description,
                &self.full_command_name,
            )?;
            self.top += 1;
            vec.push(arg);
        }
        Ok(vec)
    }

    pub fn unknown_cmd<E: From<CmdlineError>>(&mut self) -> Result<(), E> {
        let matched = self
            .last_cmd()
            .expect("invalid use of CmdlineContext::unknown_cmd - use it in a match, after calling CmdlineContext::cmd")
            .to_owned();
        if self.full_command_name.is_empty() {
            Err(E::from(CmdlineError::UnknownCommand { cmd: matched }))
        } else {
            Err(E::from(CmdlineError::UnknownSubcommand {
                cmd: self.full_command_name.clone(),
                subcmd: matched,
            }))
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
