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

Options:
  -d, --dev                  Include dev dependencies [default: excluded]
  -b, --build                Include build dependencies [default: excluded]
  -e, --exclude <WORKSPACE>  Exclude specified workspace [default: all included]
  -D, --depth <DEPTH>        The depth of dependencies to include [default: all sub dependencies]
  -h, --help                 Print help
```

## Examples
### Collect
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
├── serde_json-LICENSE-APACHE
└── serde_json-LICENSE-MIT
```
### Summary
<pre>
$ cargo licenses summary --depth 1
<strong>MIT</strong>: <span style="opacity: 0.5;">cargo_metadata</span>
<strong>MIT OR Apache-2.0</strong>: <span style="opacity: 0.5;">anyhow,clap,itertools,serde_json</span>
<strong>MPL-2.0</strong>: <span style="opacity: 0.5;">colored</span>
</pre>
```
$ cargo licenses summary --depth 1 --json
{
  "MPL-2.0": [
    "colored"
  ],
  "MIT": [
    "cargo_metadata"
  ],
  "MIT OR Apache-2.0": [
    "anyhow",
    "clap",
    "itertools",
    "serde_json"
  ]
}
```