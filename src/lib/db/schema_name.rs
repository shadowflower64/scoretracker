use regex::Regex;
use serde::{
    Deserialize, Serialize,
    de::{Unexpected, Visitor},
};
use std::{fmt, str::FromStr, sync::LazyLock};

#[derive(Debug, Clone)]
pub struct SafeSchemaName(String);

pub static SAFE_SCHEMA_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z_]{1,64}$").expect("could not compile regex"));

impl FromStr for SafeSchemaName {
    type Err = ();
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if SAFE_SCHEMA_NAME_REGEX.is_match(string) {
            Ok(Self(string.to_owned()))
        } else {
            Err(())
        }
    }
}

impl TryFrom<String> for SafeSchemaName {
    type Error = ();
    fn try_from(string: String) -> Result<SafeSchemaName, ()> {
        if SAFE_SCHEMA_NAME_REGEX.is_match(&string) {
            Ok(Self(string))
        } else {
            Err(())
        }
    }
}

impl AsRef<str> for SafeSchemaName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for &SafeSchemaName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for SafeSchemaName {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SafeSchemaName {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = SafeSchemaName;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a properly-formed safe schema name string")
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

        deserializer.deserialize_string(V)
    }
}
