# licens

A small Rust CLI that adds a license file from a built-in list. It works offline and installs with Cargo.

## Install

Once published to crates.io:

```sh
cargo install licens --locked
```

To install from a local checkout:

```sh
cargo install --path . --locked
```

## Use

```sh
licens list
licens mit
licens generate apache-2.0
licens mit --author "Jane Doe" --email "jane@example.com" --year 2026
licens mit --output docs/LICENSE.txt
```

The default output is `LICENSE` in the current directory. Missing parent directories for a custom path are created. Run `licens --help` for all options. `licens list` shows all 15 available templates; keys are case-insensitive.

If the destination already exists, licens asks whether to overwrite it. Enter, No, or an unrecognized response cancels. In noninteractive use, existing files are left untouched unless you explicitly pass `--force`:

```sh
licens mit --force
```

Directories, symbolic links, and other non-regular destinations are not overwritten, even with `--force`.

## License aliases

Save a shortcut once and use it in any project:

```sh
licens assign m=mit
licens m
licens generate m --author "Jane Doe" --output LICENSE.md
```

Aliases are case-insensitive. Names must start with an ASCII letter and contain only letters, digits, `-`, or `_`. Existing license keys, SPDX identifiers, full license names, and command names are reserved. Targets must be built-in licenses, not other aliases. Running `assign` again updates that alias without changing the others.

Aliases are stored in a small `licens/aliases.json` file under your user configuration directory: `%APPDATA%` on Windows, `~/Library/Application Support` on macOS, and `$XDG_CONFIG_HOME` (or `~/.config`) on Linux. They are not stored in your project. Built-in licenses work independently of this file.

## Author and year

Name and email are resolved separately, in this order:

1. `--author` and `--email` flags.
2. Git's effective `user.name` and `user.email` settings, if Git is installed.
3. The current directory's `package.json` author, as a string or object.
4. `GIT_AUTHOR_NAME` and `GIT_AUTHOR_EMAIL`; for the name, `USER` or `USERNAME` is also used as a fallback.

The year defaults to the current UTC year and can be set with `--year`. If no author is found for a template that uses one, licens writes an `<author>` placeholder and prints a warning. Set an author source, pass `--author`, or edit the generated file. Email is optional.

## Development

```sh
cargo test --locked
cargo build --release --locked
```

The release executable is built for size. Cargo's compiler and build cache are separate from the size of the installed executable.

## License

This project is licensed under the [MIT License](LICENSE).
