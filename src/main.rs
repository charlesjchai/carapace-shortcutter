use anyhow::{Context, Result, bail};
use clap::Parser;
use serde_json::{Map, Value};
use std::fs::{File};
use std::path::Path;
use std::io::{self, BufReader, Write};

#[cfg(not(target_family = "unix"))]
compile_error!("WHAT THE HECK ARE YOU DOING TRYING TO COMPILE THIS ON A NON-UNIX SYSTEM???");

/// Formats an error message with its location (file, line and column)
/// Do not use with functions that include an error's location
macro_rules! area_err {
    ($a:expr) => {
        format!("\"{}\" at {}:{}:{}", $a, file!(), line!(), column!())
    };
}
fn main() -> Result<()> {

    let file_path = Path::new("settings.json");
    let file: File;
    // Check if settings.json exists and create it if no
    if Path::try_exists(file_path)? {
        file = File::open(file_path)
            .context(area_err!("File could not be opened"))?;
    } else {
        file = File::create_new(file_path)
            .context(area_err!("File could not be created"))?;
    }
    let reader = BufReader::new(&file);

    // Either creates a mutable copy of settings.json or initializes an empty one
    let mut settings: Map<String, Value> = if file.metadata()?.len() == 0 {
        Map::new()
    } else {
        let json_temp = serde_json::from_reader(reader)
            .context(area_err!("Invalid JSON syntax"))?;
        match json_temp {
            Value::Object(map) => map,
            _ => bail!(area_err!("JSON root is not an object")),
        }
    };
    
    // Asks for the shell's rc file if it isn't found
    let rc_path: &Path;
    if settings.contains_key("shell_rc_file") {
        rc_path = Path::new(&settings["shell_rc_file"].to_string());
    } else {
        print!("The shell's rc file was not specified. \n\
        Please provide the path to it (~/.zshrc, ~/.bashrc), \
        or type \"auto\" to find it automatically.\n\n\
        Shell rc file path: ");
        io::stdout().flush()?;
        let mut rc_temp = String::new();
        io::stdin()
            .read_line(&mut rc_temp)
            .context(area_err!("Failed to read rc file"))?;
        rc_path = Path::new(&rc_temp);
        settings.insert("shell_rc_file".to_string(), Value::String(rc_temp.trim().to_string()));
    }
    dbg!(&settings);
    
    Ok(())
}

/// A shortcut tool that makes shortcuts of shell commands
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Add a shortcut
    add: String,
}