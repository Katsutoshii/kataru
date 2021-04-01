use crate::structs::Bookmark;
use regex::{Captures, Regex};
use std::borrow::Cow;

static VARS_RE_STR: &str = r"\$((?:[A-Za-z]+:)?(?:\w+\.)?\w+)";

lazy_static! {
    static ref VARS_RE_STRING: String = format!(r"{}\b", VARS_RE_STR);
    static ref SINGLE_VAR_RE_STRING: String = format!(r"^{}$", VARS_RE_STR);
    static ref BRACKET_VARS_RE_STRING: String = format!(r"\{{{}\}}", VARS_RE_STR);
    pub static ref VARS_RE: Regex = Regex::new(&VARS_RE_STRING).unwrap();
    pub static ref SINGLE_VAR_RE: Regex = Regex::new(&SINGLE_VAR_RE_STRING).unwrap();
    pub static ref BRACKET_VARS_RE: Regex = Regex::new(&BRACKET_VARS_RE_STRING).unwrap();
}

/// This is a line with var=${var} and var2=${var2}
pub fn replace_vars(text: &str, bookmark: &Bookmark) -> String {
    let vars_replaced = BRACKET_VARS_RE.replace_all(&text, |cap: &Captures| {
        let var = &cap[1];
        match bookmark.value(var) {
            Ok(value) => Cow::from(value.to_string()),
            Err(_) => Cow::from(format!("{{${}}}", var).to_string()),
        }
    });

    VARS_RE
        .replace_all(&vars_replaced, |cap: &Captures| {
            let var = &cap[1];
            match bookmark.value(var) {
                Ok(value) => Cow::from(value.to_string()),
                Err(_) => Cow::from(format!("${}", var).to_string()),
            }
        })
        .to_string()
}

/// Returns Some(&str) when a variable was successfully extracted.
/// Otherwise returns None.
pub fn extract_var(text: &str) -> Option<&str> {
    if SINGLE_VAR_RE.is_match(text) {
        Some(&text[1..])
    } else {
        None
    }
}

#[inline]
pub fn contains_var(text: &str) -> bool {
    VARS_RE.is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Position, Value};

    #[test]
    fn test_str_replace() {
        let bookmark = Bookmark {
            position: Position {
                namespace: "test".to_string(),
                passage: "".to_string(),
                line: 0,
            },
            state: btreemap! {
                "test".to_string() => btreemap! {
                    "var1".to_string() => Value::Number(1.0)
                },
                "global".to_string() => btreemap! {
                    "var2".to_string() => Value::String("a".to_string()),
                    "char.var1".to_string() => Value::String("b".to_string())
                }
            },
            stack: Vec::new(),
            snapshots: btreemap! {},
        };
        assert_eq!(
            replace_vars(
                "var1 = {$var1}, var2 = {$global:var2}, char.var1 = $char.var1. Tickets cost $10.",
                &bookmark
            ),
            "var1 = 1, var2 = a, char.var1 = b. Tickets cost $10."
        )
    }

    #[test]
    fn test_invalid_vars() {
        let bookmark = Bookmark::default();
        assert_eq!(
            replace_vars("var1 = ${var1}.", &bookmark),
            "var1 = ${var1}."
        )
    }
}
