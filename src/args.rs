use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about, version, author)]
pub struct CarapaceArgs {
    /// Main action
    #[command(subcommand)]
    pub command: ActionType,
}

#[derive(Debug, Subcommand)]
pub enum ActionType {
    /// Create or remove aliases
    Alias(AliasCommand),

    /// Create or remove monikers
    Moniker(MonikerCommand),

    /// Run the setup process
    Setup,

    /// TBA
    /// Cleanup junk
    Clean,
}

#[derive(Debug, Args)]
pub struct AliasCommand {
    #[command(subcommand)]
    pub subcommand: AliasSubCommand,
}
#[derive(Debug, Args)]
pub struct MonikerCommand {
    #[command(subcommand)]
    pub subcommand: MonikerSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum AliasSubCommand {
    /// Create an alias
    Add(CreateAlias),

    /// Delete an alias
    Del(RemoveAlias),

    /// List aliases
    List,
}
#[derive(Debug, Subcommand)]
pub enum MonikerSubCommand {
    /// Add a moniker
    Create(CreateMoniker),

    /// Remove a moniker
    Remove(RemoveMoniker),

    /// List monikers
    List,

    /// Run a moniker, intended for automated use (ex. `alias ls='csc moniker execute ls-fancy'`).
    Execute(ExecuteMoniker),
}

/// Running `trigger` runs `aliasee`
/// ```bash
/// csc alias create ls "ls --color=auto"
/// ls # Calls `ls --color=auto`
/// ```
#[derive(Debug, Args)]
pub struct CreateAlias {
    /// The command to activate the alias
    pub alias: String,

    /// The command you want to alias
    pub old_command: String,
}
/// The moniker needs the path of a Lua file
/// ```bash
/// touch script.lua
/// csc moniker create lua script.lua
/// ```
#[derive(Debug, Args)]
pub struct CreateMoniker {
    /// The command to activate the moniker
    pub moniker: String,

    /// The moniker's path
    pub moniker_path: PathBuf,
}

#[derive(Debug, Args)]
pub struct RemoveAlias {
    /// The alias to be removed
    pub alias: String,
}

#[derive(Debug, Args)]
pub struct RemoveMoniker {
    /// The moniker to be removed
    pub moniker: String,
}

#[derive(Debug, Args)]
pub struct ExecuteMoniker {
    /// The moniker to be executed (name WITHOUT .lua file extention)
    pub moniker: String,

    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}
