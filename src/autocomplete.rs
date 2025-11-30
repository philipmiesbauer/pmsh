use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};
use std::borrow::Cow;

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
