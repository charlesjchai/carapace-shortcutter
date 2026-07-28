use anyhow::{Context, Result};
use clap::Parser;
use serde_json::Value;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{PathBuf};

#[cfg(not(target_family = "unix"))]
compile_error!("Support for non-unix systems is not yet supported.");

fn main() -> Result<()> {
    assert!(
        cfg!(target_family = "unix"),
        "Support for non-unix systems is not yet supported."
    );

    let file_path = PathBuf::from("settings.json");
    // Open the file and create it if it doesn't exist
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path.to_str().unwrap())?;
    /*let mut file: File =  if Path::try_exists(file_path)? {
        File::open(file_path).context(area_err!("File could not be opened"))?
    } else {
        File::create_new(file_path).context(area_err!("File could not be created"))?
    };*/
    let reader = BufReader::new(&file);

    // Either creates a mutable copy of settings.json or initializes an empty one
    let mut json_val = if file.metadata()?.len() == 0 {
        Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_reader(reader).context(area_err!("Invalid syntax"))?
    };
    let settings = json_val
        .as_object_mut()
        .context(area_err!("Not an object"))?;
    /*let mut settings: Map<String, Value> = if file.metadata()?.len() == 0 {
        Map::new()
    } else {
        let json_temp =
            serde_json::from_reader(reader).context(area_err!("Invalid JSON syntax"))?;
        match json_temp {
            Value::Object(map) => map,
            _ => bail!(area_err!("JSON root is not an object")),
        }
    };*/

    // Asks for the shell's rc file if it isn't found
    let rc_path: PathBuf;
    let mut buf = String::new();
    if settings.contains_key("shell_rc_file") {
        rc_path = PathBuf::from(
            settings["shell_rc_file"]
                .as_str()
                .context("Error reading JSON")?,
        );
    } else {
        print!(
            "The shell's rc file was not specified. \n\
        Please provide the path to it (~/.zshrc, ~/.bashrc), \
        or type \"auto\" to find it automatically.\n\n\
        Shell rc file path: "
        );
        io::stdout().flush()?;
        io::stdin()
            .read_line(&mut buf)
            .context(area_err!("Failed to read rc file"))?;
        rc_path = PathBuf::from(&buf);
        settings.insert(
            "shell_rc_file".to_string(),
            Value::String(buf.trim().to_string()),
        );
    }
    dbg!(&settings);

    let shellrc_file = File::options()
        .read(true)
        .write(true)
        .append(true)
        .open(rc_path);

    // Truncate the file and quit
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);
    quit(&mut writer, &json_val);
}

/// Formats an error message with its location (file, line and column)
/// Only use with anyhow error handling
macro_rules! area_err {
    ($a:expr) => {
        format!("\"{}\" at {}:{}:{}", $a, file!(), line!(), column!())
    };
}
pub(crate) use area_err;

/// A shortcut tool that makes shortcuts of shell commands
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Add a shortcut
    add: String,
}

fn quit(mut writer: impl std::io::Write, value: &Value) -> ! {
    serde_json::to_writer_pretty(&mut writer, value).expect("Could not read Value");
    writer.flush().unwrap();
    std::process::exit(0);
}
