
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

## Contributing

PRs welcome! Please ensure all tests pass and code is formatted.

## License

MIT
