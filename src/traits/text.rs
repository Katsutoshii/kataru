use crate::error::ParseError;
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Trait for parsable expressions.
pub trait FromYaml<'a>: DeserializeOwned {
    fn from_yml(text: &'a str) -> Result<Self, ParseError> {
        match serde_yaml::from_str(text) {
            Ok(config) => Ok(config),
            Err(e) => Err(perror!("Invalid YAML: {}", e)),
        }
    }
}

/// Trait for parsable expressions.
pub trait FromStr<'a> {
    fn from_str(text: &'a str) -> Result<Self, ParseError>
    where
        Self: std::marker::Sized;
}

/// Trait for extract config/story from MessagePack bytes.
pub trait FromMessagePack<'a>: Deserialize<'a> {
    fn from_mp(bytes: &'a [u8]) -> Self {
        rmp_serde::from_slice(bytes).unwrap()
    }
}
