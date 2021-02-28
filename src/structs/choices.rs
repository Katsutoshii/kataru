use super::{Bookmark, Conditional, Map};
use crate::traits::FromStr;
use crate::{error::Result, traits::MoveValues};
use linear_map::LinearMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawChoice {
    Conditional(Map<String, Option<String>>),
    PassageName(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RawChoices {
    pub choices: LinearMap<String, RawChoice>,
    #[serde(default)]
    pub timeout: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Choices {
    pub choices: Map<String, String>,
    #[serde(default)]
    pub timeout: f64,
}

impl Choices {
    pub fn from(other: &mut Self) -> Result<Self> {
        Ok(Self {
            choices: Map::move_values(&mut other.choices)?,
            timeout: other.timeout,
        })
    }

    pub fn get_valid(raw: &RawChoices, bookmark: &Bookmark) -> Result<Self> {
        let mut valid = Self {
            choices: Map::default(),
            timeout: raw.timeout,
        };
        let mut passage = "";

        // Populate valid choices and infer implicit passage targets.
        for (key, choice) in raw.choices.iter().rev() {
            match choice {
                // Populate top level choices.
                RawChoice::PassageName(Some(passage_name)) => {
                    passage = passage_name;
                    valid
                        .choices
                        .insert(key.to_string(), passage_name.to_string());
                }
                RawChoice::PassageName(None) => {
                    valid.choices.insert(key.to_string(), passage.to_string());
                }
                // Populate all choices are behind a true conditional.
                RawChoice::Conditional(conditional) => {
                    if !Conditional::from_str(key)?.eval(&bookmark)? {
                        continue;
                    }
                    for (choice_text, passage_name_opt) in conditional.iter().rev() {
                        if let Some(passage_name) = passage_name_opt {
                            passage = passage_name;
                            valid
                                .choices
                                .insert(choice_text.to_string(), passage_name.to_string());
                        } else {
                            valid.choices.insert(key.to_string(), passage.to_string());
                        }
                    }
                }
            }
        }
        Ok(valid)
    }
}
