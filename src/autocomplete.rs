use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};
use std::borrow::Cow;
use crate::completion_registry::COMP_REGISTRY;

fn extract_command_and_word(line: &str, pos: usize) -> (String, String) {
    let current_line = &line[..pos];
    let mut words = current_line.split_whitespace().collect::<Vec<&str>>();

    if words.is_empty() {
        return (String::new(), String::new());
    }

    let command = words[0].to_string();

    let word_being_completed = if current_line.ends_with(char::is_whitespace) {
        String::new()
    } else {
        words.pop().unwrap().to_string()
    };

    if words.is_empty() {
        return (String::new(), command);
    }

    (command, word_being_completed)
}

pub struct PmshHelper {
    pub completer: FilenameCompleter,
}

impl PmshHelper {
    pub fn new() -> Self {
        Self {
            completer: FilenameCompleter::new(),
        }
    }
}

impl Completer for PmshHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        let (command, word_being_completed) = extract_command_and_word(line, pos);

        if !command.is_empty() {
            if let Ok(registry) = COMP_REGISTRY.read() {
                if let Some(spec) = registry.get(&command) {
                    if let Some(wordlist) = spec.wordlist {
                        let mut matches = Vec::new();
                        let words: Vec<&str> = wordlist.split_whitespace().collect();
                        for w in words {
                            if w.starts_with(&word_being_completed) {
                                matches.push(Pair {
                                    display: w.to_string(),
                                    replacement: w.to_string(),
                                });
                            }
                        }

                        if !matches.is_empty() || word_being_completed.is_empty() {
                            let start_pos = pos - word_being_completed.len();
                            return Ok((start_pos, matches));
                        }
                    }
                }
            }
        }

        // Fallback to file completion
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for PmshHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for PmshHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        false
    }
}

impl Validator for PmshHelper {}

impl Helper for PmshHelper {}
