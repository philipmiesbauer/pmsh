pub const RESET: &str = "\x1b[0m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";

pub fn red(s: &str) -> String {
    format!("{}{}{}", RED, s, RESET)
}

pub fn green(s: &str) -> String {
    format!("{}{}{}", GREEN, s, RESET)
}

pub fn yellow(s: &str) -> String {
    format!("{}{}{}", YELLOW, s, RESET)
}

pub fn blue(s: &str) -> String {
    format!("{}{}{}", BLUE, s, RESET)
}

pub fn cyan(s: &str) -> String {
    format!("{}{}{}", CYAN, s, RESET)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_red() {
        assert_eq!(red("test"), "\x1b[31mtest\x1b[0m");
    }

    #[test]
    fn test_green() {
        assert_eq!(green("test"), "\x1b[32mtest\x1b[0m");
    }
}
