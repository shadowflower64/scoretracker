use std::{fmt, path::PathBuf, str::FromStr};

use scoretracker::data::library::stpl_url::{LibraryDomain, LibraryDomainName};

use crate::error::CmdError;

pub struct ArgError {
    error_message: String,
}

impl ArgError {
    pub fn from_parse_err<T: fmt::Display>(e: T) -> Self {
        return Self {
            error_message: e.to_string(),
        };
    }
}

pub trait CmdlineArgumentParse: Sized {
    fn try_from_arg(arg: &String) -> Result<Self, ArgError>;
}

pub trait CmdlineArgument: CmdlineArgumentParse {
    fn arg_type() -> &'static str;
}

pub fn parse_arg<T: CmdlineArgument>(arg: Option<&String>, name: &str, description: &str, fcn: &str) -> Result<T, CmdError> {
    if let Some(arg) = arg {
        T::try_from_arg(arg).map_err(|e| CmdError::WrongArgumentType {
            cmd: fcn.to_string(),
            arg_name: name.to_string(),
            arg_desc: description.to_string(),
            arg_type: T::arg_type().to_string(),
            err_msg: e.error_message,
        })
    } else {
        Err(CmdError::ArgumentNotProvided {
            cmd: fcn.to_string(),
            arg_name: name.to_string(),
            arg_desc: description.to_string(),
        })
    }
}

pub fn parse_arg_opt<T: CmdlineArgument>(arg: Option<&String>, name: &str, description: &str, fcn: &str) -> Result<Option<T>, CmdError> {
    if let Some(arg) = arg {
        T::try_from_arg(arg).map(Some).map_err(|e| CmdError::WrongArgumentType {
            cmd: fcn.to_string(),
            arg_name: name.to_string(),
            arg_desc: description.to_string(),
            arg_type: T::arg_type().to_string(),
            err_msg: e.error_message,
        })
    } else {
        Ok(None)
    }
}

impl<T> CmdlineArgumentParse for T
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Display,
{
    fn try_from_arg(arg: &String) -> Result<Self, ArgError> {
        arg.parse().map_err(ArgError::from_parse_err)
    }
}

impl CmdlineArgument for PathBuf {
    fn arg_type() -> &'static str {
        "path"
    }
}

impl CmdlineArgument for String {
    fn arg_type() -> &'static str {
        "string"
    }
}

impl CmdlineArgument for f64 {
    fn arg_type() -> &'static str {
        "float (f64)"
    }
}

impl CmdlineArgument for bool {
    fn arg_type() -> &'static str {
        "boolean"
    }
}

impl CmdlineArgument for LibraryDomainName {
    fn arg_type() -> &'static str {
        "valid library domain name"
    }
}

impl CmdlineArgument for LibraryDomain {
    fn arg_type() -> &'static str {
        "valid library domain"
    }
}
