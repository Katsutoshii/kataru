use crate::vars::replace_vars;
use crate::{
    structs::{
        Bookmark, Branchable, Choice, Dialogue, Line, Passage, QualifiedName, State,
        StateUpdatable, Story, StoryGetters,
    },
    Value,
};

pub struct Runner<'r> {
    pub bookmark: &'r mut Bookmark,
    pub story: &'r Story,
    pub line_num: usize,
    pub passage: &'r Passage,
    lines: Vec<&'r Line>,
    line: Line,
    breaks: Vec<usize>,
    speaker: &'r str,
}

impl<'r> Runner<'r> {
    pub fn new(bookmark: &'r mut Bookmark, story: &'r Story) -> Self {
        // Flatten dialogue lines
        let passage = &story
            .passage(&QualifiedName::from(&bookmark.namespace, &bookmark.passage))
            .unwrap();
        let mut runner = Self {
            bookmark,
            story,
            line_num: 0,
            lines: vec![],
            line: Line::Continue,
            passage,
            breaks: vec![],
            speaker: "",
        };
        runner.load_lines(passage);
        runner.init_breaks();
        runner
    }

    /// Initialize the line break stack.
    /// Loop through each line in the flattened array until current line
    /// number is reached.
    /// Each time a branch is detected, push the end of the branch on the break stack.
    fn init_breaks(&mut self) {
        for (line_num, line) in self.lines.iter().enumerate() {
            if line_num >= self.bookmark.line {
                break;
            }
            match line {
                Line::Break => {
                    self.breaks.pop();
                }
                Line::Branches(branches) => {
                    self.breaks.push(line_num + branches.len());
                }
                _ => (),
            }
        }
    }

    /// Loads lines into a single flat array of references.
    fn load_lines(&mut self, lines: &'r [Line]) {
        for line in lines {
            match line {
                Line::Branches(branches) => {
                    self.lines.push(&line);

                    // Add breaks after each line except for the last line
                    let mut branches_it = branches.iter();
                    if let Some((_expression, branch_lines)) = branches_it.next() {
                        self.load_lines(branch_lines);
                    }
                    for (_expression, branch_lines) in branches_it {
                        self.lines.push(&Line::Break);
                        self.load_lines(branch_lines);
                    }
                }
                _ => self.lines.push(&line),
            }
        }
    }

    /// Goto a given `passage_name`.
    fn goto(&mut self, passage_name: &str) {
        // `passage_name` could be:
        // 1) a local name (unquallified), in which case namespace stays the same.
        // 2) a qualified name pointing to another section, in which case we switch namespace.
        // 3) a global name, in which we must changed namespace to root.
        let qname = QualifiedName::from(&self.bookmark.namespace, passage_name);
        self.passage = match self
            .story
            .get(&qname.namespace)
            .unwrap()
            .passage(&qname.name)
        {
            Some(passage) => {
                if !qname.namespace.is_empty() {
                    self.bookmark.namespace = qname.namespace;
                }
                passage
            }
            None => {
                self.bookmark.namespace = "".to_string();
                self.story.get("").unwrap().passage(&qname.name).unwrap()
            }
        };

        self.bookmark.passage = qname.name;
        self.bookmark.line = 0;

        self.lines = vec![];
        self.breaks = vec![];
        self.load_lines(self.passage);
    }

    /// Processes a line.
    /// Returning Line::Continue signals to `next()` that another line should be processed
    /// before returning a line to the user.
    fn process_line(&mut self, input: &str, line: &'r Line) -> Line {
        match line {
            // When a choice is encountered, it should first be returned for display.
            // Second time it's encountered, go to the chosen passage.
            Line::Choices(choices) => {
                // If empty input, chocies are being returned for display.
                if input.is_empty() {
                    Line::Choices(choices.get_valid(&self.bookmark))
                } else if let Line::Choices(ref mut choices) = self.line {
                    if choices.choices.contains_key(input) {
                        if let Some(Choice::PassageName(passage_name)) =
                            choices.choices.remove(input)
                        {
                            self.goto(&passage_name);
                        }
                        Line::Continue
                    } else {
                        Line::InvalidChoice
                    }
                } else {
                    Line::Error
                }
            }
            // When input is encountered, it should first be returned for display.
            // Second time it's encountered, modify state.
            Line::InputCmd(input_cmd) => {
                if input.is_empty() {
                    line.clone()
                } else {
                    for (var, _prompt) in &input_cmd.input {
                        let mut state = State::new();
                        state.insert(var.clone(), Value::String(input.to_string()));
                        let root_sets = self.bookmark.state().update(&state).unwrap();
                        self.bookmark.root_state().update(&root_sets).unwrap();
                    }
                    self.bookmark.line += 1;
                    Line::Continue
                }
            }
            Line::Branches(branches) => {
                let skipped_len = branches.take(&mut self.bookmark).unwrap();
                let branch_len = branches.length();
                self.breaks
                    .push(self.bookmark.line + branch_len - skipped_len);
                Line::Continue
            }
            Line::Goto(goto) => {
                self.goto(&goto.goto);
                Line::Continue
            }
            Line::Break => {
                let last_break = self.breaks.pop();
                self.bookmark.line = match last_break {
                    Some(line_num) => line_num,
                    None => 0,
                };
                Line::Continue
            }
            Line::Cmds(_) => {
                self.bookmark.line += 1;
                line.clone()
            }
            Line::SetCmd(set) => {
                let root_sets = self.bookmark.state().update(&set.set).unwrap();
                self.bookmark.root_state().update(&root_sets).unwrap();
                self.bookmark.line += 1;
                Line::Continue
            }
            Line::Dialogue(dialogue) => {
                let mut replaced_dialogue = Dialogue::new();
                for (character, text) in dialogue {
                    self.speaker = character;
                    replaced_dialogue
                        .insert(character.to_string(), replace_vars(text, self.bookmark));
                }
                self.bookmark.line += 1;
                Line::Dialogue(replaced_dialogue)
            }
            Line::Text(text) => {
                let mut dialogue = Dialogue::new();
                dialogue.insert(self.speaker.to_string(), replace_vars(text, self.bookmark));
                self.bookmark.line += 1;
                Line::Dialogue(dialogue)
            }
            Line::Continue => {
                self.bookmark.line += 1;
                Line::Continue
            }
            Line::End => Line::End,
            Line::Error => Line::Error,
            Line::InvalidChoice => Line::InvalidChoice,
        }
    }

    /// If the current configuration points to a valid line, processes the line.
    fn process(&mut self, input: &str) -> Line {
        if self.bookmark.line >= self.lines.len() {
            Line::Error
        } else {
            self.process_line(input, self.lines[self.bookmark.line])
        }
    }

    /// Gets the next dialogue line from the story based on the user's input.
    /// Internally, a single call to `next()` may result in multiple lines being processed,
    /// i.e. when a choice is being made.
    pub fn next(&mut self, input: &str) -> &Line {
        self.line = self.process(input);
        while self.line == Line::Continue {
            self.line = self.process("");
        }
        &self.line
    }
}
