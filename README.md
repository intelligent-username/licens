# licens

A CLI tool that generates project licenses and audits your dependencies' licenses in one pass.

## What it does

`licens` handles two related tasks:

**Generate a LICENSE file** for your project by prompting for your name, email, and license choice. It looks for author info in your git config, environment variables, or package.json, or CLI settings if available.

**Audit dependency licenses** by scanning your Cargo.toml, pyproject.toml, and other package manager files and fetching license information from crates.io. The output shows which licenses your dependencies use and can flag ones that conflict with your project's license. It also recommends the most permissive/restrictive license for your project based on the licenses of your dependencies.

## Installation

Coming soon. Once the project is finished, you will be able to install it using cargo:

```bash
cargo install licens
```

## Usage

Generate a LICENSE file:

```bash
licens mit # Adds an MIT license to your project
```

Audit your project's dependency licenses:
```bash
licens audit
```

List available licenses:
```bash
licens list
```

## Supported licenses

- MIT
- GPL-2.0
- GPL-3.0
- Apache-2.0
- BSD-1-Clause
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Unlicense
- WTFPL

## Flags

`--format json` — Output audit results as JSON instead of markdown
`--dev` — Include dev dependencies in audit
`--build` — Include build dependencies in audit
`--output <path>` — Write LICENSE file to a custom location

## License

This project is licensed under the MIT License.
