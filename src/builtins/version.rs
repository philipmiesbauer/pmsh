use crate::parser::SimpleCommand;

pub fn execute(cmd: &SimpleCommand) -> Result<(), String> {
    if !cmd.args.is_empty() {
        return Err(format!("{}: no arguments expected", cmd.name));
    }

    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");

    println!("{} version {}", name, version);
    println!("A simple shell written in Rust.");
    println!("GitHub: https://github.com/philipmiesbauer/pmsh");

    Ok(())
}
