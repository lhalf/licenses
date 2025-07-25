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
Usage: cargo licenses [OPTIONS] <COMMAND>

Commands:
  folder   Collects all licenses into a folder
  summary  Provides a summary of all licenses

Options:
  -d, --dev                  Include dev dependencies [default: excluded]
  -b, --build                Include build dependencies [default: excluded]
  -e, --exclude <WORKSPACE>  Exclude specified workspace [default: all included]
  -D, --depth <DEPTH>        The depth of dependencies to collect licenses for [default: all sub dependencies]
  -h, --help                 Print help
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
```
$ cargo licenses --depth 1 summary
MIT
MIT OR Apache-2.0
MPL-2.0
```
