use crate::error::ParseError;
use crate::structs::{CharacterData, Config, Params, Passage, Passages, Value};
use crate::traits::{Loadable, Mergeable, Parsable};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;

/// A qualified name is a name in an explicit namespace.
/// If namespace is empty, then this name is global.
pub struct QualifiedName {
    pub namespace: String,
    pub name: String,
}

impl QualifiedName {
    pub fn from(namespace: &str, name: &str) -> Self {
        let split: Vec<&str> = name.rsplitn(2, ":").collect();
        match split.as_slice() {
            [split_name, explicit_namespace] => Self {
                namespace: explicit_namespace.to_string(),
                name: split_name.to_string(),
            },
            _ => Self {
                namespace: namespace.to_string(),
                name: name.to_string(),
            },
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub config: Config,
    pub passages: Passages,
}

impl<'a> Section {
    pub fn passage(&'a self, name: &str) -> Option<&'a Passage> {
        self.passages.get(name)
    }
    pub fn params(&'a self, name: &str) -> Option<&'a Option<Params>> {
        self.config.cmds.get(name)
    }
    pub fn character(&'a self, name: &str) -> Option<&'a CharacterData> {
        self.config.characters.get(name)
    }
    pub fn value(&'a self, name: &str) -> Option<&'a Value> {
        self.config.state.get(name)
    }
}

impl Mergeable for Section {
    fn merge(&mut self, other: &mut Self) -> Result<(), ParseError> {
        self.config.merge(&mut other.config)?;
        self.passages.merge(&mut other.passages)?;
        Ok(())
    }
}

impl Loadable for Section {
    fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let source = Self::load_string(path)?;
        let split: Vec<&str> = source.split("---").collect();
        if let [config_str, passages_str] = &split[1..] {
            Ok(Self {
                config: Config::parse(config_str).unwrap(),
                passages: Passages::parse(passages_str).unwrap(),
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Unable to parse file.",
            ))
        }
    }
}