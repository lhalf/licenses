# licenses

[![crates.io](https://img.shields.io/crates/v/licenses)](https://crates.io/crates/licenses)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/lhalf/licenses/on_commit.yml)](https://github.com/lhalf/licenses/actions/workflows/on_commit.yml)
[![MIT](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

Cargo subcommand for collecting licenses.

## Install

```bash
$ cargo install licenses
```

## Usage

```
$ cargo licenses --help
Usage: cargo licenses [OPTIONS] <COMMAND>

Commands:
  collect  Collects all licenses into a folder
  summary  Provides a summary of all licenses
  check    Checks all licenses for inconsistencies

Options:
  -d, --dev                  Include dev dependencies [default: excluded]
  -b, --build                Include build dependencies [default: excluded]
  -D, --depth <DEPTH>        The depth of dependencies to include [default: all sub dependencies]
  -e, --exclude <WORKSPACE>  Exclude specified workspace [default: all included]
  -i, --ignore <CRATE>       Ignore specified crate [default: all included]
  -h, --help                 Print help
```

## Commands

### Collect

Collects all licenses of the specified dependencies into a folder.

Prints a warning:

- If the crate had no declared license on [crates.io](https://crates.io/)
- If no licenses were found for a crate
- If the found licenses did not match those declared by the author on crates.io
- If the content of the found licenses did not match the expected content for that license

```bash
$ cargo licenses collect --depth 1
```

```
licenses
├── anyhow-LICENSE-APACHE
├── anyhow-LICENSE-MIT
├── cargo_metadata-LICENSE-MIT
├── clap-LICENSE-APACHE
├── clap-LICENSE-MIT
├── colored-LICENSE
├── itertools-LICENSE-APACHE
├── itertools-LICENSE-MIT
├── once_cell-LICENSE-APACHE
├── once_cell-LICENSE-MIT
├── serde-LICENSE-APACHE
├── serde-LICENSE-MIT
├── serde_json-LICENSE-APACHE
├── serde_json-LICENSE-MIT
├── spdx-LICENSE-APACHE
├── spdx-LICENSE-MIT
└── strsim-LICENSE
```

### Summary

Summarises the declared licenses of the specified dependencies.

```
$ cargo licenses summary --depth 1
MIT: cargo_metadata,strsim
MIT OR Apache-2.0: anyhow,clap,itertools,once_cell,serde,serde_json,spdx
MPL-2.0: colored
```

```
$ cargo licenses summary --depth 1 --json
{
  "MIT OR Apache-2.0": [
    "anyhow",
    "clap",
    "itertools",
    "once_cell",
    "serde",
    "serde_json",
    "spdx"
  ],
  "MIT": [
    "cargo_metadata",
    "strsim"
  ],
  "MPL-2.0": [
    "colored"
  ]
}

```

### Check

Checks all licenses of the specified dependencies for inconsistencies.

Returns a non-zero exit code:

- If the crate had no declared license on [crates.io](https://crates.io/)
- If no licenses were found for a crate
- If the found licenses did not match those declared by the author on crates.io
- If the content of the found licenses did not match the expected content for that license

```
$ cargo licenses check
warning: found license(s) in memchr whose content was not similar to expected - COPYING
warning: found license(s) in unicode_xid whose content was not similar to expected - COPYRIGHT
warning: found license(s) in utf8_iter whose content was not similar to expected - COPYRIGHT
```