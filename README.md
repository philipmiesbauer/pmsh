
# pmsh

Philip Miesbauer SHell (pmsh) — a modern, minimal shell written in Rust.

[![Codecov](https://codecov.io/gh/philipmiesbauer/pmsh/branch/main/graph/badge.svg)](https://codecov.io/gh/philipmiesbauer/pmsh)

## Features

- Interactive REPL with line editing (rustyline)
- Command parsing and execution (external commands)
- Builtins: `cd`, `cd -`, `history`, `exit`
- Persistent command history (`~/.pmsh_history`, up to 1000 entries)
- Prompt shows user and current directory, with `~` for HOME
- Tilde expansion and collapse for paths
- Deterministic, robust unit and integration tests
- CI with code quality and coverage reporting

## Usage

Build and run:

```bash
cargo build --release
./target/release/pmsh
```

### Example session

```bash
philip:~$ echo hello
hello
philip:~$ cd /tmp
philip:/tmp$ cd /var
philip:/var$ cd -
/tmp
philip:/tmp$ history
1: echo hello
2: cd /tmp
3: cd /var
4: cd -
5: history
philip:/tmp$ exit
Exiting.
```

## Builtins

- `cd [dir]` — change directory (supports `~` and `cd -` for previous dir)
- `history` — print command history
- `exit` — save history and exit

## History

- Commands are saved to `~/.pmsh_history` (up to 1000 entries)
- History is loaded on startup and saved on exit and after each command

## Prompt

- Format: `<user>:<cwd>$ `
- HOME is shown as `~` (e.g., `philip:~$`)

## Development

Run tests:

```bash
cargo test
```

Run integration test (PTY-based):

```bash
cargo test --test integration_repl
```

Check formatting and lints:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
```

Generate coverage (requires cargo-tarpaulin):

```bash
cargo tarpaulin --out Xml --out Lcov --run-types Tests
```

## Internal Tools Help

This section documents pmsh’s internal modules and how to use them in code and tests.

- Parser (`src/parser.rs`)
	- Purpose: Parse a command line into a `Command { name, args }`.
	- API: `Command::parse(line: &str) -> Option<Command>`
	- Example:
		```rust
		use crate::parser::Command;
		if let Some(cmd) = Command::parse("echo hello world") {
				assert_eq!(cmd.name, "echo");
				assert_eq!(cmd.args, vec!["hello".into(), "world".into()]);
		}
		```

- Executor (`src/executor.rs`)
	- Purpose: Execute external programs with arguments.
	- API: `Executor::execute(cmd: &Command) -> Result<(), String>`
	- Example:
		```rust
		use crate::executor::Executor;
		use crate::parser::Command;
		let cmd = Command { name: "echo".into(), args: vec!["hi".into()] };
		Executor::execute(&cmd)?;
		```

- History Manager (`src/history.rs`)
	- Purpose: Persist and manage command history (`~/.pmsh_history`).
	- APIs:
		- `HistoryManager::new() -> Result<Self, String>`
		- `load(&self) -> Result<Vec<String>, String>`
		- `save(&self, history: &[String]) -> Result<(), String>`
		- `add_entry(&self, entry: &str, history: &mut Vec<String>) -> Result<(), String>`
	- Example:
		```rust
		use crate::history::HistoryManager;
		let mgr = HistoryManager::new()?;
		let mut hist = mgr.load()?;
		mgr.add_entry("echo hi", &mut hist)?;
		mgr.save(&hist)?;
		```

- Path Utilities (`src/path_utils.rs`)
	- Purpose: Home expansion/compaction.
	- APIs:
		- `expand_home(path: &str) -> String` (turn `$HOME/...` into `~/...` for display)
		- `collapse_tilde(path: &str) -> std::path::PathBuf` (turn `~/...` into absolute path)
	- Example:
		```rust
		use crate::path_utils::{expand_home, collapse_tilde};
		let shown = expand_home("/home/user/projects");
		let abs = collapse_tilde("~/projects");
		```

- UI (`src/ui.rs`)
	- Purpose: Prompt formatting.
	- APIs:
		- `format_prompt() -> String`
		- `format_prompt_with(cwd: &str, user: &str) -> String` (pure, test helper)
	- Example:
		```rust
		use crate::ui::format_prompt_with;
		let p = format_prompt_with("/home/user", "alice");
		assert!(p.starts_with("alice:"));
		```

- Builtins (`src/builtins.rs`)
	- Purpose: Handle internal shell commands.
	- API: `handle_builtin(cmd, history_mgr, command_history, oldpwd) -> Result<BuiltinResult, String>`
		- Supports `cd [dir]`, `cd -`, `history`, `exit`
		- `BuiltinResult::{HandledContinue, HandledExit, NotHandled}`
	- Example:
		```rust
		use crate::builtins::{handle_builtin, BuiltinResult};
		use crate::history::HistoryManager;
		use crate::parser::Command;
		let mgr = HistoryManager::new()?;
		let mut hist = vec![];
		let mut oldpwd = None;
		let cmd = Command { name: "cd".into(), args: vec!["/tmp".into()] };
		match handle_builtin(&cmd, &mgr, &mut hist, &mut oldpwd)? {
				BuiltinResult::HandledContinue => {}
				_ => {}
		}
		```

- REPL (`src/repl.rs`)
	- Purpose: Event loop driving input, builtins, execution, and history.
	- APIs:
		- `run_repl(editor, history_mgr, command_history, executor)`
		- Traits: `LineEditor` (readline, add_history_entry), `ExecutorTrait` (execute)
		- Adapter: `RealExecutor` bridges to `Executor`
	- Example (testing with mocks):
		```rust
		use crate::repl::{run_repl, ReadlineEvent, LineEditor, ExecutorTrait};
		# struct MockEditor; # struct MockExec; # /* see tests for full mocks */
		# impl LineEditor for MockEditor { /* ... */ }
		# impl ExecutorTrait for MockExec { /* ... */ }
		# let mut editor = MockEditor; let exec = MockExec; let mgr = Default::default();
		let mut history = vec![];
		run_repl(&mut editor, &mgr, &mut history, &exec);
		```

- Integration Tests (`tests/integration_repl.rs`)
	- Purpose: End-to-end validation via a PTY using `expectrl`.
	- How to run:
		```bash
		cargo test --test integration_repl
		```

## Contributing

PRs welcome! Please ensure all tests pass and code is formatted.

## License

MIT
