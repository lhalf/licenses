# licenses

[![crates.io](https://img.shields.io/crates/v/licenses)](https://crates.io/crates/licenses)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/lhalf/licenses/on_commit.yml)](https://github.com/lhalf/licenses/actions/workflows/on_commit.yml)
[![MIT](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

Cargo subcommand for collecting, summarising and checking licenses.

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

Collects all licenses for the specified dependencies. Will alert the following license discrepancies:
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

Summarises the declared licenses for the specified dependencies.

<pre>
$ cargo licenses summary --depth 1
<strong>MIT</strong>: <span style="opacity: 0.5;">cargo_metadata,strsim</span>
<strong>MIT OR Apache-2.0</strong>: <span style="opacity: 0.5;">anyhow,clap,itertools,once_cell,serde,serde_json,spdx</span>
<strong>MPL-2.0</strong>: <span style="opacity: 0.5;">colored</span>
</pre>
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