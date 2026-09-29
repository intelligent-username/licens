# licens

A fast, developer-friendly CLI tool that generates project licenses and audits your dependencies' licenses in one pass.

## What it does

`licens` handles two core tasks:

1. **Generate a `LICENSE` file** for your project. Automatically resolves your name, email, and current year from Git config, `package.json`, environment variables, or CLI overrides.
2. **Audit dependency licenses** across project manifests (`Cargo.toml`, `package.json`, `pyproject.toml`). It inspects dependencies, queries registry data (such as crates.io), flags license conflicts (e.g. copyleft vs. permissive), and recommends optimal license choices.

---

## Installation

Install using `cargo`:

```bash
cargo install licens
```

Or build and run locally from source:

```bash
cargo run -- <command>
```

---

## Usage Guide

### 1. Generate a LICENSE file

To create a license file in your current directory:

```bash
# Add an MIT license (default creates 'LICENSE')
licens mit

# Add an Apache 2.0 license
licens apache-2.0

# Explicit command syntax
licens generate gpl-3.0
```

#### Custom Output Location
```bash
# Write to a custom path or file name
licens mit --output LICENSE.md
licens bsd-3-clause --output docs/LICENSE.txt
```

#### Author & Year Overrides
By default, `licens` detects your information automatically. You can explicitly override any field:

```bash
licens mit --author "Jane Doe" --email "jane@example.com" --year 2026
```

---

### 2. Audit Project Dependencies

Scan manifests in the current directory and audit all dependencies for licensing and compatibility:

```bash
# Audit standard dependencies and output as Markdown
licens audit
```

#### Include Development and Build Dependencies
```bash
# Audit runtime, dev, and build dependencies
licens audit --dev --build
```

#### Output Formats
By default, `licens audit` prints a clean Markdown table and summary. You can output JSON for CI/CD pipelines or scripts:

```bash
# JSON output
licens audit --format json

# Pipe JSON to jq
licens audit --format json | jq '.conflicts'
```

---

### 3. List Supported Licenses

View all available license templates, their SPDX identifiers, and license categories:

```bash
licens list
```

Example output:
```text
KEY             SPDX ID         CATEGORY           NAME
----------------------------------------------------------------------
mit             MIT             Permissive         MIT License
apache-2.0      Apache-2.0      Permissive         Apache License 2.0
gpl-2.0         GPL-2.0         Strong Copyleft    GNU General Public License v2.0
gpl-3.0         GPL-3.0         Strong Copyleft    GNU General Public License v3.0
bsd-1-clause    BSD-1-Clause    Permissive         BSD 1-Clause License
bsd-2-clause    BSD-2-Clause    Permissive         BSD 2-Clause License
bsd-3-clause    BSD-3-Clause    Permissive         BSD 3-Clause License
isc             ISC             Permissive         ISC License
unlicense       Unlicense       Public Domain      The Unlicense
wtfpl           WTFPL           Public Domain      Do What The Fuck You Want To Public License
```

---

## Author Detection Order

When generating a license, `licens` discovers author details using the following precedence:

1. **CLI Flags**: `--author <name>`, `--email <email>`, `--year <year>`
2. **Git Configuration**: `[user]` section in `.git/config` or `~/.gitconfig` (`name` and `email`)
3. **`package.json`**: `author` field (string `"Name <email>"` or object `{"name": "...", "email": "..."}`)
4. **Environment Variables**: `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL`, `USER`, or `USERNAME`
5. **System Fallbacks**: Defaults to current UTC year and `<author>` if unresolvable.

---

## Supported Manifests

`licens audit` automatically detects and scans:

- **Rust**: `Cargo.toml` (including Cargo workspace inheritance and `[workspace.dependencies]`)
- **Node.js**: `package.json` (`dependencies`, `devDependencies`)
- **Python**: `pyproject.toml` (PEP 621 `[project.dependencies]` and Poetry `[tool.poetry.dependencies]`)

---

## Conflict Detection & Recommendations

During an audit, `licens` analyzes compatibility between your project's chosen license and the licenses of your dependencies:

- **Viral Copyleft Warnings**: Flags when a permissive project (e.g., MIT, BSD) depends on strong copyleft code (e.g., GPL-2.0, GPL-3.0, AGPL-3.0).
- **Incompatible Licenses**: Flags specific legal incompatibilities (e.g., Apache-2.0 patent clauses combined with GPL-2.0).
- **License Recommendations**: Calculates both the **most permissive** license and the **most restrictive** license your project can adopt without legal conflicts.

---

## CLI Options & Flags

| Command / Flag | Description | Default |
| :--- | :--- | :--- |
| `licens <key>` | Generate license with template key (e.g. `mit`, `gpl-3.0`) | - |
| `licens generate <key>` | Explicit syntax to generate a license | - |
| `licens audit` | Run dependency license audit | - |
| `licens list` | Print supported license templates table | - |
| `--output <path>`, `-o` | Target file path for license generation | `LICENSE` |
| `--author <name>` | Override author name | *Auto-detected* |
| `--email <email>` | Override author email | *Auto-detected* |
| `--year <year>` | Override copyright year | *Current Year* |
| `--format <format>`, `-f` | Audit output format: `markdown` or `json` | `markdown` |
| `--dev` | Include development dependencies in audit | `false` |
| `--build` | Include build dependencies in audit | `false` |

---

## License

This project is licensed under the [MIT License](LICENSE).
