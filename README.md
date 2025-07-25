# licenses

[![crates.io](https://img.shields.io/crates/v/licenses)](https://crates.io/crates/licenses)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/lhalf/licenses/on_commit.yml)]((https://github.com/lhalf/licenses/actions/workflows/on_commit.yml))
[![MIT](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

Command line tool for collecting licenses.

## Install

```bash
cargo install licenses
```

## Usage

```
$ cargo licenses --help
Command line tool for collecting licenses.

Usage: licenses [OPTIONS] <COMMAND>

Commands:
  folder  Collects all licenses into a folder
  help    Print this message or the help of the given subcommand(s)

Options:
  -d, --dev                  Include dev dependencies [default: excluded]
  -b, --build                Include build dependencies [default: excluded]
  -e, --exclude <WORKSPACE>  Exclude specified workspace [default: all included]
  -D, --depth <DEPTH>        The depth of dependencies to collect licenses for [default: all sub dependencies]
  -h, --help                 Print help
  -V, --version              Print version
```

## Example

```bash
cargo licenses --depth 1 folder
```
```
licenses
├── anyhow-LICENSE-APACHE
├── anyhow-LICENSE-MIT
├── cargo_metadata-LICENSE-MIT
├── clap-LICENSE-APACHE
├── colored-LICENSE
├── itertools-LICENSE-APACHE
└── itertools-LICENSE-MIT
```