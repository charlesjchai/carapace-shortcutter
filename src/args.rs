use clap::{Args, Parser};

/// A shortcut tool that makes shortcuts of shell commands
#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CarapaceArgs {
    /// Add a shortcut
    pub add: String,
}