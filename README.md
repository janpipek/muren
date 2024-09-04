# muren(mu)ltiple (ren)ames

Command-line utility for filename manipulations.

```
Usage: muren [OPTIONS] [COMMAND]

Commands:
  set-ext    Change extension
  prefix     Prefix with string
  replace    Replace parts of the name
  normalize  Convert names to reasonable ASCII.
  fix-ext    Fix extension according to the file contents.
  remove     Remove part of a name from all files.
  help       Print this message or the help of the given subcommand(s)

Options:
  -d, --dry      Dry run
  -y, --yes      Automatically confirm all actions
  -h, --help     Print help
  -V, --version  Print version
```

## Installation

Once you have [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html) on your system:

```
cargo install muren
```

## Alternatives

- [rnr](https://github.com/ismaelgv/rnr)

Why am I implementing this then? Because this project scratches my particular itch and allows me to try/learn rust.