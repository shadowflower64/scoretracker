use crate::prompt_user;
use std::fmt::Display;
use std::io::{self, Write};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
#[error("could not read string from stdin: {0}")]
pub struct AskError(#[from] io::Error);

pub fn ask<T: Display>(prompt: &str, fallback: Option<T>, validator: impl Fn(&str) -> Result<Option<T>, ()>) -> Result<T, AskError> {
    loop {
        if let Some(default_value) = fallback.as_ref() {
            prompt_user!("{prompt} [{default_value}]: ");
        } else {
            prompt_user!("{prompt}: ");
        }

        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;

        if let Ok(parsed) = validator(&buf) {
            if let Some(value) = parsed {
                return Ok(value);
            } else {
                if let Some(default_value) = fallback {
                    return Ok(default_value);
                }
            }
        }
    }
}

pub fn ask_string(prompt: &str, fallback: Option<String>) -> Result<String, AskError> {
    ask(prompt, fallback, |answer| {
        let answer = answer.trim();
        if !answer.is_empty() {
            Ok(Some(answer.to_string()))
        } else {
            Ok(None)
        }
    })
}

pub fn ask_yn(prompt: &str, fallback: Option<bool>) -> Result<bool, AskError> {
    ask(&format!("{prompt} (y/n)"), fallback, |answer| {
        let answer = answer.trim().to_lowercase();
        if answer == "y" {
            Ok(Some(true))
        } else if answer == "n" {
            Ok(Some(false))
        } else if answer.is_empty() {
            Ok(None)
        } else {
            Err(())
        }
    })
}

pub fn ask_parse<T: FromStr + Display>(prompt: &str, fallback: Option<T>) -> Result<T, AskError> {
    ask(prompt, fallback, |answer| {
        let answer = answer.trim();
        if answer.is_empty() {
            Ok(None)
        } else {
            answer.parse().map(Some).map_err(|_| ())
        }
    })
}

pub fn ask_bool(prompt: &str, fallback: Option<bool>) -> Result<bool, AskError> {
    ask_parse(prompt, fallback)
}

pub fn ask_i64(prompt: &str, fallback: Option<i64>) -> Result<i64, AskError> {
    ask_parse(prompt, fallback)
}

pub fn ask_u64(prompt: &str, fallback: Option<u64>) -> Result<u64, AskError> {
    ask_parse(prompt, fallback)
}

pub fn ask_f64(prompt: &str, fallback: Option<f64>) -> Result<f64, AskError> {
    ask_parse(prompt, fallback)
}

pub fn ask_uuid(prompt: &str, fallback: Option<Uuid>) -> Result<Uuid, AskError> {
    ask_parse(prompt, fallback)
}
