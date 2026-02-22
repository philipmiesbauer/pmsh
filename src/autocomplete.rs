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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completion_registry::{CompSpec, COMP_REGISTRY};

    #[test]
    fn test_extract_command_and_word() {
        let (cmd, word) = extract_command_and_word("ls ", 3);
        assert_eq!(cmd, "ls");
        assert_eq!(word, "");

        let (cmd, word) = extract_command_and_word("ls -", 4);
        assert_eq!(cmd, "ls");
        assert_eq!(word, "-");

        let (cmd, word) = extract_command_and_word("git checko", 10);
        assert_eq!(cmd, "git");
        assert_eq!(word, "checko");

        // When there's only one word (no command prefix), returns ("", word)
        let (cmd, word) = extract_command_and_word("", 0);
        assert_eq!(cmd, "");
        assert_eq!(word, "");

        // If line is just one word, command is "empty"
        let (cmd, word) = extract_command_and_word("ls", 2);
        assert_eq!(cmd, "");
        assert_eq!(word, "ls");
    }

    #[test]
    fn test_extract_git_context() {
        let res = extract_git_context("git checkout ", 13);
        assert!(res.is_some());
        let (subcmd, words) = res.unwrap();
        assert_eq!(subcmd, "checkout");
        assert_eq!(words, 2);

        let res = extract_git_context("git push origin ", 16);
        assert!(res.is_some());
        let (subcmd, words) = res.unwrap();
        assert_eq!(subcmd, "push");
        assert_eq!(words, 3); // git, push, origin

        let res = extract_git_context("ls -l", 5);
        assert!(res.is_none());

        // Mid-typing a subcommand (no trailing whitespace)
        let res = extract_git_context("git chec", 8);
        // "chec" is popped off, so words = ["git"] which has len < 2
        assert!(res.is_none());

        // Mid-typing after subcommand
        let res = extract_git_context("git checkout mai", 16);
        // "mai" popped, words = ["git", "checkout"] => len == 2
        assert!(res.is_some());
        let (subcmd, _) = res.unwrap();
        assert_eq!(subcmd, "checkout");
    }

    #[test]
    fn test_get_git_branches_and_remotes() {
        // These call git but we're not inside a git repo + can't assert the output,
        // so just confirm they don't panic.
        let _branches = get_git_branches();
        let _remotes = get_git_remotes();
    }

    #[test]
    fn test_pmsh_helper_completer() {
        let _helper = PmshHelper::new();
        // Context instantiation requires private fields.
        // Core logic is tested through the extract_* functions above.
    }

    #[test]
    fn test_pmsh_helper_hinter_highlighter() {
        let helper = PmshHelper::new();
        assert_eq!(helper.highlight("ls", 2), Cow::Borrowed("ls"));
        assert!(!helper.highlight_char("ls", 2, false));
    }

    #[test]
    #[serial_test::serial]
    fn test_complete_with_registry() {
        // Register a wordlist for "test_cmd"
        if let Ok(mut registry) = COMP_REGISTRY.write() {
            registry.register(
                "test_cmd".to_string(),
                CompSpec {
                    wordlist: Some("alpha beta gamma".to_string()),
                },
            );
        }

        // Now use a rustyline Editor to get a real Context for testing complete
        let helper = PmshHelper::new();
        let config = rustyline::Config::default();
        let h = rustyline::Editor::<PmshHelper, rustyline::history::DefaultHistory>::with_config(config).unwrap();
        let history = h.history();
        let ctx = rustyline::Context::new(history);

        // "test_cmd " - command has no word being completed
        let result = helper.complete("test_cmd ", 9, &ctx);
        assert!(result.is_ok());
        let (_, pairs) = result.unwrap();
        // All words match empty prefix
        assert_eq!(pairs.len(), 3);

        // "test_cmd al" - should filter to "alpha"
        let result = helper.complete("test_cmd al", 11, &ctx);
        assert!(result.is_ok());
        let (_, pairs) = result.unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].display, "alpha");

        // Cleanup
        if let Ok(mut registry) = COMP_REGISTRY.write() {
            registry.remove("test_cmd");
        }
    }
}
