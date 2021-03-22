use colored::*;
use kataru::*;
use linear_map::LinearMap;
use regex::{Captures, Regex};
use std::{
    borrow::Cow,
    io::{stdin, stdout, Write},
};

#[macro_use]
extern crate lazy_static;

fn get_input(input: &mut String) {
    let _ = stdout().flush();
    *input = String::new();
    stdin().read_line(input).expect("Invalid input");
    loop {
        if let Some('\n') = input.chars().next_back() {
            input.pop();
        } else if let Some('\r') = input.chars().next_back() {
            input.pop();
        } else {
            break;
        }
    }
}

fn await_key(input: &mut String) {
    get_input(input);
    *input = String::new();
}

/// Validate the story.
/// Return true iff story is valid.
#[cfg(debug_assertions)]
fn print_validation(story: &Story) -> bool {
    println!("{}", "Validating story...".bold().cyan());
    match Validator::new(story).validate() {
        Err(e) => {
            println!("{}", format!("{}", e).red());
            false
        }
        Ok(_) => {
            println!("{}", "Validated story successfully.".bold().green());
            true
        }
    }
}

fn run_command(runner: &mut Runner, command: &str, _params: &LinearMap<String, Value>) {
    match command {
        "ClearScreen" => print!("{}[2J", 27 as char),
        "SaveSnapshot" => {
            runner.save_snapshot("test");
            println!("Snapshot saved.");
            println!("{}", format!("{:#?}", runner.bookmark.snapshots).italic());
        }
        "LoadSnapshot" => {
            runner.load_snapshot("test").unwrap();
            println!("Snapshot loaded.");
            println!("{}", format!("{:#?}", runner.bookmark.stack).italic());
        }
        _ => println!("{}", format!("{}: {:#?}", command, _params).italic()),
    }
}

fn replace_tags_ansi(text: &str) -> String {
    lazy_static! {
        static ref TAGS_RE: Regex = Regex::new(r"<([:a-zA-Z0-9_\./]*)>").unwrap();
    }
    TAGS_RE
        .replace_all(&text, |cap: &Captures| {
            let tag = &cap[1];
            let code = match tag {
                "b" => "\x1b[1m",
                "/b" => "\x1b[0m",
                _ => "",
            };
            Cow::from(code.to_string())
        })
        .to_string()
}

fn handle_line(runner: &mut Runner, input: &mut String) -> bool {
    // println!("{:#?}", runner.bookmark);
    let line = runner.next(&input).unwrap().clone();
    match &line {
        Line::Dialogue(dialogue) => {
            match dialogue.name.as_str() {
                "Narrator" => print!("{}", replace_tags_ansi(&dialogue.text).italic()),
                _ => print!(
                    "{}: {}",
                    dialogue.name.bold().yellow(),
                    replace_tags_ansi(&dialogue.text)
                ),
            }
            await_key(input);
            true
        }
        Line::Choices(choices) => {
            println!();
            for (choice, _passage_name) in &choices.choices {
                println!("{}", choice.cyan());
            }
            print!("\n{}", "Enter your choice: ".bold().magenta());
            get_input(input);
            true
        }
        Line::Command(cmd) => {
            for (command, params) in cmd {
                run_command(runner, command, params);
            }
            true
        }
        Line::Input(input_cmd) => {
            for (_var, prompt) in &input_cmd.input {
                print!("{}: ", prompt.bold().magenta());
                get_input(input);
            }
            true
        }
        Line::InvalidChoice => {
            print!(
                "{}",
                format!("Invalid choice '{}', try again: ", input).magenta()
            );
            get_input(input);
            true
        }
        Line::End => {
            println!("End of story.");
            false
        }
        _ => {
            println!("Invalid line encountered: {:?}", line);
            false
        }
    }
}

fn main() {
    // Load the story.
    println!("{}", "Loading story...".bold().cyan());
    let mut bookmark = Bookmark::from_mp(include_bytes!("../target/bookmark")).unwrap();
    let story = Story::from_mp(include_bytes!("../target/story")).unwrap();

    #[cfg(debug_assertions)]
    if !print_validation(&story) {
        return;
    }

    bookmark.init_state(&story);
    let mut runner = Runner::new(&mut bookmark, &story).unwrap();
    let mut input = String::new();

    while handle_line(&mut runner, &mut input) {}
}
