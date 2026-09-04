# carapace-shortcutter (csc)

## Description

carapace-shortcutter is a command-line utility that allows you to make shortcuts of existing commands. Unlike the `alias` command, it allows you manage your aliases in one spot, without opening your shell's rc file. However, it is in the early phases, so many features are not yet implemented.

## Run it

### Building from source

To build from source, you will need the following dependencies:

- `git`
- `cargo`
- A Unix-based operating system (macOS, Linux, BSD), Windows is not yet supported.

Then, run the following commands:

```
git clone https://github.com/charlesjchai/carapace-shortcutter
cd carapace-shortcutter
cargo install --path .
```

### Using it

With carapace-shortcutter, you can currently create, remove, or list aliases.

To add an alias, run `csc alias add <ALIAS> <OLD_COMMAND>` For example, to make the `ls` command colorful, run `csc alias add ls "ls --color=auto"`. To remove an alias, run `csc alias del <ALIAS>`. To remove that `ls` alias, run `csc alias del ls`.

To list the current aliases, simply `run csc alias list`.

## License
This project is licensed under the MIT License, see [LICENSE](LICENSE) for more details.
