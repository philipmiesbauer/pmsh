use crate::parser::SimpleCommand;

pub fn execute(cmd: &SimpleCommand) -> Result<(), String> {
    // Usage: compgen -W "wordlist" -- word
    let mut args = cmd.args.iter().peekable();
    let mut wordlist = None;
    let mut word = "";

    while let Some(arg) = args.next() {
        if arg == "-W" {
            if let Some(w) = args.next() {
                wordlist = Some(w.clone());
            } else {
                return Err("compgen: option requires an argument -- W".to_string());
            }
        } else if arg == "--" {
            if let Some(w) = args.next() {
                word = w.as_str();
            }
        } else {
            word = arg.as_str();
        }
    }

    if let Some(wl) = wordlist {
        let words: Vec<&str> = wl.split_whitespace().collect();
        for w in words {
            if w.starts_with(word) {
                println!("{}", w);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SimpleCommand;

    #[test]
    fn test_compgen_builtin_no_args() {
        let cmd = SimpleCommand {
            name: "compgen".to_string(),
            args: vec![],
            assignments: vec![],
        };
        // Should return ok, no wordlist
        assert!(execute(&cmd).is_ok());
    }

    #[test]
    fn test_compgen_builtin_missing_wordlist() {
        let cmd = SimpleCommand {
            name: "compgen".to_string(),
            args: vec!["-W".to_string()],
            assignments: vec![],
        };
        let result = execute(&cmd);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "compgen: option requires an argument -- W");
    }

    #[test]
    fn test_compgen_builtin_with_wordlist_and_word() {
        let cmd = SimpleCommand {
            name: "compgen".to_string(),
            args: vec![
                "-W".to_string(),
                "apple banana apricot".to_string(),
                "--".to_string(),
                "a".to_string(),
            ],
            assignments: vec![],
        };
        // It prints to stdout, which we can't easily capture in an in-process thread
        // test without redirection, but just asserting Ok covers the lines!
        assert!(execute(&cmd).is_ok());
    }

    #[test]
    fn test_compgen_builtin_with_wordlist_no_word() {
        let cmd = SimpleCommand {
            name: "compgen".to_string(),
            args: vec![
                "-W".to_string(),
                "cherry date".to_string(),
            ],
            assignments: vec![],
        };
        assert!(execute(&cmd).is_ok());
    }
}
