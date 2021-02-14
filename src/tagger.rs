use crate::structs::Line;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineTag {
    Choices,
    InvalidChoice,
    Dialogue,
    InputCmd,
    Cmds,
    None,
}

impl LineTag {
    pub fn tag(line_opt: &Option<Line>) -> Self {
        match line_opt {
            Some(line) => match line {
                Line::Choices(_) => LineTag::Choices,
                Line::Dialogue(_) => LineTag::Dialogue,
                Line::Cmds(_) => LineTag::Cmds,
                Line::InputCmd(_) => LineTag::InputCmd,
                Line::InvalidChoice => LineTag::InvalidChoice,
                _ => LineTag::None,
            },
            None => LineTag::None,
        }
    }
}