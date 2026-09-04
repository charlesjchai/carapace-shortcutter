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

    /// Synchronize the `shortcuts` file with the data.json file
    Synchronize,

    /// Run the setup process
    Setup,
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
    /// Add an alias
    Create(CreateAlias),

    /// Remove an alias
    Remove(RemoveObject),
}
#[derive(Debug, Subcommand)]
pub enum MonikerSubCommand {
    /// Add a moniker
    Create(CreateMoniker),

    /// Remove a moniker
    Remove(RemoveObject),
}

/// Running `trigger` runs `aliasee`
/// ```
/// csc alias create ls "ls --color=auto"
/// ls # Calls `ls --color=auto`
/// ```
#[derive(Debug, Args)]
pub struct CreateAlias {
    /// The command to activate the alias
    pub trigger: String,

    /// The command you want to alias
    pub aliasee: String,
}
/// The moniker needs the path of a Lua file
/// ```
///
/// ```
#[derive(Debug, Args)]
pub struct CreateMoniker {
    /// The command to activate the moniker
    pub trigger: String,

    /// The moniker's path
    pub moniker_path: PathBuf,
}

/// NOTE: Aliases and monikers share the same remove struct
#[derive(Debug, Args)]
pub struct RemoveObject {
    /// The object to be removed
    pub shortcut: String,
}
