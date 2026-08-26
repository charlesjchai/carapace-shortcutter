use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about, version, author)]
pub struct CarapaceArgs {
    /// Main action
    #[command(subcommand)]
    pub command: ObjectType,
}

#[derive(Debug, Subcommand)]
pub enum ObjectType {
    /// Create or remove aliases
    Alias(AliasCommand),

    /// Create or remove monikers
    Moniker(MonikerCommand),

    /// Run the setup process
    Setup,
}

#[derive(Debug, Args)]
pub struct AliasCommand {
    #[command(subcommand)]
    pub command: AliasSubCommand,
}
#[derive(Debug, Args)]
pub struct MonikerCommand {
    #[command(subcommand)]
    pub command: MonikerSubCommand,
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

/// "alias" and "aliasee" are equivalent to the left and right sides of the builtin alias command.
/// ```
/// alias ls='ls --color=auto'
/// pacecut alias create ls "ls --color=auto"
/// ```
#[derive(Debug, Args)]
pub struct CreateAlias {
    /// The command to activate the alias
    pub trigger: String,

    /// The command you want to alias
    pub aliasee: String,
}
/// The moniker needs the path of a Lua file
#[derive(Debug, Args)]
pub struct CreateMoniker {
    /// The command to activate the moniker
    pub trigger: String,

    /// The moniker's path
    pub moniker_path: PathBuf,
}

#[derive(Debug, Args)]
pub struct RemoveObject {
    /// The object to be removed
    pub object: String,
}
