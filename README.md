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
  diff     Diff the current licenses folder against what would be collected

Options:
  -d, --dev                  Include dev dependencies [default: excluded]
  -b, --build                Include build dependencies [default: excluded]
  -D, --depth <DEPTH>        The depth of dependencies to include [default: all sub dependencies]
  -e, --exclude <WORKSPACE>  Exclude specified workspace [default: all included]
  -i, --ignore <CRATE>       Ignore specified crate [default: all included]
  -c, --config <PATH>        Path to configuration file
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
├── strsim-LICENSE
├── toml-LICENSE-APACHE
└── toml-LICENSE-MIT
```

### Summary

Summarises the declared licenses of the specified dependencies.

```
$ cargo licenses summary --depth 1
MIT: cargo_metadata,strsim
MIT OR Apache-2.0: anyhow,clap,itertools,once_cell,serde,serde_json,spdx,toml
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
    "spdx",
    "toml"
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

```
$ cargo licenses summary --depth 1 --toml
"MIT OR Apache-2.0" = [
    "anyhow",
    "clap",
    "itertools",
    "once_cell",
    "serde",
    "serde_json",
    "spdx",
    "toml",
]
MIT = [
    "cargo_metadata",
    "strsim",
]
"MPL-2.0" = ["colored"
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

## Configuration

A [TOML](https://toml.io/en/) configuration file can be used to store all passed flags, as well as enabling some additional features.
If both a config and a flag are passed, the flag will take precedence.

### Skipping licenses

The configuration file allows the selective skipping of licenses found by the various subcommands.
It is recommended to provide a comment per skipped license to indicate why it is deemed safe to skip, for instance it might be
erroneously detected as a license because of the filename.

### Example

The below is an example of a TOML configuration file that could be used via the `--config` flag.

```toml
[global]
dev = true
build = true
depth = 1
exclude = ["workspace"]
ignore = ["crate"]

[crate.memchr]
skipped = ["COPYING"] # not a license, statement of which licenses the crate falls under

[crate.unicode-xid]
skipped = ["COPYRIGHT"] # not a license, statement of which licenses the crate falls under

[crate.utf8_iter]
skipped = ["COPYRIGHT"] # not a license, statement of which licenses should be used
```

## Usage patterns

This tool is designed to help collect required licenses when shipping software with open-source dependencies. The intended pattern of use would look as follows:

- `summary` provides a quick way to see if any dependencies are using stricter licenses that might not be suitable
- `collect` to collect all licenses into an output folder, this would be done manually and the license folder commited as part of the repository
- the previous command might have raised warnings about licenses found, or not found, these can be manually assessed and skipped in the configuration file if deemed safe
- as part of a continuous integration system, or as a pre-commit hook, a `diff` should be run to check the licenses folder hasn't missed any licenses added by new dependencies
- as part of a continuous integration system a `check` should be run to confirm all license inconsistencies have been accounted for

## Legal disclaimer

This is provided as a convenience to help with collecting and reviewing open-source licenses. **It does not guarantee compliance with all legal licensing requirements.** It is
the user's responsibility to ensure that all applicable licenses are collected, reviewed and adhered to. The authors and contributors of this tool accept no liability for missing,
incomplete or inaccurate licenses files, or for any consequences arising from its use.