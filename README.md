# muren(mu)ltiple (ren)ames

Command-line utility for filename manipulations.

```
Usage: muren [OPTIONS] [COMMAND]

Commands:
  set-ext      Change extension
  prefix       Prefix with string
  replace      Replace parts of the name
  normalize    Convert names to reasonable ASCII.
  fix-ext      Fix extension according to the file contents.
  remove       Remove part of a name from all files.
  change-case  Change case of all files.
  help         Print this message or the help of the given subcommand(s)

Options:
  -d, --dry        Dry run
  -u, --unchanged  Show unchanged files
  -y, --yes        Automatically confirm all actions
  -h, --help       Print help
  -V, --version    Print version
```

## Why not [rnr](https://github.com/ismaelgv/rnr)?

Because this project scratches my particular itch and allows me to try/learn rust.
