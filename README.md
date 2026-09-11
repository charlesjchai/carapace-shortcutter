# carapace-shortcutter (csc)

## Description

carapace-shortcutter is a command-line utility that allows you to make shortcuts of existing commands. Unlike the `alias` command, it allows you manage your aliases with one command. Additionally, you can add aliases for Lua scripts (called 'monikers') for deeper automation, all without opening your shell's rc file!

## Run it

### Building from source

To build from source, you will need the following dependencies:

- `git`
- `cargo`
- A Unix-based operating system (macOS, Linux, BSD), you must use WSL to run the program on Windows.

Then, run the following commands:

```
git clone https://github.com/charlesjchai/carapace-shortcutter
cd carapace-shortcutter
cargo install --path .
```

### Using it

With carapace-shortcutter, you can create, remove, or list aliases. You can also do the same with Lua scripts, named monikers.

#### Aliases

To add an alias, run `csc alias add <ALIAS> <OLD_COMMAND>` For example, to make the `ls` command colorful, run `csc alias add ls "ls --color=auto"`. To remove an alias, run `csc alias del <ALIAS>`. To remove that `ls` alias, run `csc alias del ls`.

To list the current aliases, simply run `csc alias list`.

#### Monikers

Monikers are shortcuts to Lua scripts. You can add one by typing `csc moniker create <MONIKER> <MONIKER_PATH>`. You can remove or list them in a similar way to aliases, by typing `csc moniker remove <MONIKER>`, and `csc moniker list`

## License
This project is licensed under the MIT License, see [LICENSE](LICENSE) for more details.
