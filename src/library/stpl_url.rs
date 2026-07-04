use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum StplUrlError {
    #[error("protocol is not present in url")]
    ProtocolNotPresent,
    #[error("url protocol must be 'stpl://' (was '{0}://')")]
    InvalidProtocol(String),
    #[error("library domain name cannot contain character: '{0}'")]
    DomainNameContainsChar(char),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LibraryDomainName(String);

impl TryFrom<String> for LibraryDomainName {
    type Error = StplUrlError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.contains("/") {
            return Err(StplUrlError::DomainNameContainsChar('/'));
        }
        Ok(Self(value))
    }
}

impl Display for LibraryDomainName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for LibraryDomainName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
struct LibraryDomainNameVisitor;

impl<'de> Visitor<'de> for LibraryDomainNameVisitor {
    type Value = LibraryDomainName;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a properly-formed url domain name string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.to_owned().try_into().map_err(|_x| E::invalid_value(Unexpected::Str(v), &Self))
    }

    // fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    // where
    //     E: serde::de::Error,
    // {
    //     v.to_owned().try_into().map_err(|_x| E::invalid_value(Unexpected::Str(v), &Self))
    // }
}

impl<'de> Deserialize<'de> for LibraryDomainName {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_string(LibraryDomainNameVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LibraryDomain {
    Local(LibraryDomainName),
    Global(LibraryDomainName),
}

impl TryFrom<String> for LibraryDomain {
    type Error = StplUrlError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.rsplit_once(".local") {
            Some((prefix, _suffix)) => {
                let domain_name = prefix.to_owned().try_into()?;
                Ok(Self::Local(domain_name))
            }
            None => {
                todo!();
                // let domain_name = value.try_into()?;
                // Ok(Self::Global(domain_name))
            }
        }
    }
}

impl Display for LibraryDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global(_identifier) => todo!(), //write!(f, "{identifier}"),
            Self::Local(identifier) => write!(f, "{identifier}.local"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StplUrl {
    pub domain: LibraryDomain,
    pub path: Option<String>,
}

impl StplUrl {
    pub fn new(domain: LibraryDomain, path: Option<String>) -> Self {
        Self { domain, path }
    }

    pub fn try_parse(string: &str) -> Result<Self, StplUrlError> {
        Self::try_from(string.to_owned())
    }

    pub fn try_parse_parts(domain: &str, path: &str) -> Result<Self, StplUrlError> {
        Ok(Self {
            domain: domain.to_owned().try_into()?,
            path: Some(path.to_owned()),
        })
    }

    pub fn try_parse_without_path(domain: &str) -> Result<Self, StplUrlError> {
        Ok(Self {
            domain: domain.to_owned().try_into()?,
            path: None,
        })
    }
}

impl TryFrom<String> for StplUrl {
    type Error = StplUrlError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (protocol, rest) = value.split_once("://").ok_or(StplUrlError::ProtocolNotPresent)?;
        if protocol != "stpl" {
            return Err(StplUrlError::InvalidProtocol(protocol.to_owned()));
        }

        if let Some((domain_string, path)) = rest.split_once('/') {
            let domain = domain_string.to_owned().try_into()?;
            Ok(Self {
                domain,
                path: Some(path.to_owned()),
            })
        } else {
            let domain = rest.to_owned().try_into()?;
            Ok(Self { domain, path: None })
        }
    }
}

impl Display for StplUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.path {
            Some(path) => write!(f, "stpl://{}/{}", self.domain, path),
            None => write!(f, "stpl://{}", self.domain),
        }
    }
}

impl Serialize for StplUrl {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

struct StplUrlVisitor;

impl<'de> Visitor<'de> for StplUrlVisitor {
    type Value = StplUrl;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a properly-formed 'stpl://' url string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.to_owned().try_into().map_err(|_x| E::invalid_value(Unexpected::Str(v), &Self))
    }

    // fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    // where
    //     E: serde::de::Error,
    // {
    //     v.to_owned().try_into().map_err(|_x| E::invalid_value(Unexpected::Str(v), &Self))
    // }
}

impl<'de> Deserialize<'de> for StplUrl {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_string(StplUrlVisitor)
    }
}
