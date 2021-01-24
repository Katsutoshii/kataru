use crate::structs::Bookmark;
use regex::{Captures, Regex};
use std::borrow::Cow;

/// This is a line with var=${var} and var2=${var2}
pub fn replace_vars(text: &str, bookmark: &Bookmark) -> String {
    lazy_static! {
        static ref VARS_RE: Regex = Regex::new(r"\$\{([:a-zA-Z0-9_\.]*)\}").unwrap();
    }
    VARS_RE
        .replace_all(&text, |cap: &Captures| {
            let var = &cap[1];
            match bookmark.value(var) {
                Some(value) => Cow::from(value.to_string()),
                None => Cow::from(format!("${{{}}}", var).to_string()),
            }
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Map, Value};

    #[test]
    fn test_str_replace() {
        let mut bookmark = Bookmark::default();
        bookmark.namespace = "test".to_string();

        bookmark.state.insert("test".to_string(), Map::new());
        let test_state = bookmark.state.get_mut("test").unwrap();
        test_state.insert("var1".to_string(), Value::Number(1.0));

        bookmark.state.insert("".to_string(), Map::new());
        let root_state = bookmark.state.get_mut("").unwrap();
        root_state.insert("var2".to_string(), Value::String("a".to_string()));
        root_state.insert("char.var1".to_string(), Value::String("b".to_string()));

        assert_eq!(
            replace_vars(
                "var1 = ${var1}, var2 = ${:var2}, char.var1 = ${char.var1}. Tickets cost $10.",
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
