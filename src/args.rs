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
    /// Create, remove, or update aliases
    Alias(AliasCommand),

    /// Run the setup process
    Setup,
}

#[derive(Debug, Args)]
pub struct AliasCommand {
    #[command(subcommand)]
    pub command: AliasSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum AliasSubCommand {
    /// Add a shortcut
    Create(CreateAlias),

    /// Remove a shortcut
    Remove(RemoveObject),
}

#[derive(Debug, Args)]
pub struct CreateAlias {
    /// The name of your shortcut
    pub name: String,

    /// The command you want to alias
    pub old_command: String,

    /// The final alias
    pub alias: String,
}

#[derive(Debug, Args)]
pub struct RemoveObject {
    /// The object to be removed
    pub entity: String,
}
