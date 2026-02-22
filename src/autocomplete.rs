use crate::completion_registry::COMP_REGISTRY;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};
use std::borrow::Cow;
use std::process::Command;

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

fn get_git_branches() -> Vec<String> {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            return stdout.lines().map(|s| s.to_string()).collect();
        }
    }
    Vec::new()
}

fn get_git_remotes() -> Vec<String> {
    let output = Command::new("git").arg("remote").output();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            return stdout.lines().map(|s| s.to_string()).collect();
        }
    }
    Vec::new()
}

// Extracts the subcommand, and the number of words before the current word
fn extract_git_context(line: &str, pos: usize) -> Option<(String, usize)> {
    let current_line = &line[..pos];
    let mut words = current_line.split_whitespace().collect::<Vec<&str>>();
    if current_line.ends_with(char::is_whitespace) {
        // if we have trailing whitespace, the user is starting a *new* word.
        // "words" already has all completed preceding words.
    } else {
        // the user is mid-typing a word, pop it off to just look at context
        words.pop();
    }

    if words.len() >= 2 && words[0] == "git" {
        return Some((words[1].to_string(), words.len()));
    }
    None
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
            // Check for native dynamic git completion first
            if command == "git" && !word_being_completed.starts_with('-') {
                if let Some((subcommand, num_words)) = extract_git_context(line, pos) {
                    let mut candidates = Vec::new();

                    if num_words == 2 {
                        // "git <subcmd> <TAB>"
                        match subcommand.as_str() {
                            "checkout" | "switch" | "merge" | "rebase" | "branch" => {
                                candidates = get_git_branches();
                            }
                            "push" | "pull" | "fetch" | "remote" => {
                                candidates = get_git_remotes();
                            }
                            _ => {}
                        }
                    } else if num_words == 3 {
                        // "git <subcmd> <arg1> <TAB>"
                        match subcommand.as_str() {
                            "push" | "pull" | "fetch" => {
                                // If the command interacts with a remote and we already typed the remote,
                                // the next argument is usually a branch.
                                candidates = get_git_branches();
                            }
                            _ => {}
                        }
                    }

                    if !candidates.is_empty() {
                        let mut matches = Vec::new();
                        for w in candidates {
                            if w.starts_with(&word_being_completed) {
                                matches.push(Pair {
                                    display: w.clone(),
                                    replacement: w,
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

            // Fallback to static COMPLETION REGISTRY
            // For git, we also check if there's a specific `git-<subcommand>` registered (e.g. `complete -W ... git-checkout`)
            let registry_key = if command == "git" {
                if let Some((subcmd, _)) = extract_git_context(line, pos) {
                    format!("git-{}", subcmd)
                } else {
                    command.clone()
                }
            } else {
                command.clone()
            };

            if let Ok(registry) = COMP_REGISTRY.read() {
                if let Some(spec) = registry
                    .get(&registry_key)
                    .or_else(|| registry.get(&command))
                {
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
