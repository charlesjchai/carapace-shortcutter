use clap::{Args, Parser, Subcommand};

/// A shortcut tool that makes shortcuts of shell commands
#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CarapaceArgs {
    /// Main action
    #[clap(subcommand)]
    pub command: CommandType,
}

#[derive(Debug, Subcommand)]
pub enum CommandType {
    /// Create, remove, or update shortcuts
    Add,
}
