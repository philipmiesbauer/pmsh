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
