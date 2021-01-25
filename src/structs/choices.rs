use super::{Bookmark, Conditional, Map};
use crate::traits::Parsable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Choice {
    Conditional(Map<String, String>),
    PassageName(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Choices {
    pub choices: Map<String, Choice>,
    #[serde(default)]
    pub timeout: f64,
}

impl Choices {
    pub fn get_valid(&self, bookmark: &Bookmark) -> Self {
        let mut valid = Self::default();
        for (key, choice) in &self.choices {
            match choice {
                Choice::PassageName(passage_name) => {
                    valid.choices.insert(
                        key.to_string(),
                        Choice::PassageName(passage_name.to_string()),
                    );
                }
                Choice::Conditional(conditional) => {
                    for (choice_text, passage_name) in conditional {
                        if !Conditional::parse(key).unwrap().eval(&bookmark).unwrap() {
                            continue;
                        }
                        valid.choices.insert(
                            choice_text.to_string(),
                            Choice::PassageName(passage_name.to_string()),
                        );
                    }
                }
            }
        }
        valid
    }
}