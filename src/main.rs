mod args;

use anyhow::{Context, Result};
use args::CarapaceArgs;
use clap::Parser;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{self, Path, PathBuf};
#[cfg(not(unix))]
compile_error!("Non-Unix systems are not yet supported.");

fn main() -> Result<()> {
    let json_path = PathBuf::from("resources/settings.json");
    let aliases_path = PathBuf::from("resources/aliases");
    let json_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(json_path.to_str().unwrap())
        .context(area_err!("Could not open settings.json"))?;
    let mut aliases_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o755)
        .open(aliases_path.to_str().unwrap())
        .context(area_err!("Could not open `aliases` file"))?;
    // writeln!(aliases_file, "ls")?;
    let reader = BufReader::new(&json_file);

    // Either creates a mutable copy of settings.json or initializes an empty one
    let mut json_val = if json_file.metadata()?.len() == 0 {
        Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_reader(reader).context(area_err!("Invalid syntax"))?
    };
    let settings_json = json_val
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
    let shellrc_path: PathBuf;
    let mut buf = String::new();
    if settings_json.contains_key("shellrc_file") {
        shellrc_path = PathBuf::from(
            settings_json["shellrc_file"]
                .as_str()
                .context(area_err!("Error reading JSON"))?,
        );
    } else {
        print!(
            "The shell's rc file was not specified. \n\
        Please provide the ABSOLUTE path to it ({}), \
        or type \"auto\" to find it automatically.\n\n\
        > ",
            if cfg!(target_os = "macos") {
                "/Users/<user>/.bashrc"
            } else {
                "/home/<user>/.bashrc"
            }
        );
        io::stdout().flush()?;
        io::stdin()
            .read_line(&mut buf)
            .context(area_err!("Failed to read rc file"))?;
        shellrc_path = PathBuf::from(buf.trim());
        settings_json.insert(
            "shellrc_file".to_string(),
            Value::String(buf.trim().to_string()),
        );
    }
    dbg!(&settings_json);
    /*

    let shellrc_file = File::options()
        .read(true)
        .write(true)
        .append(true)
        .open(shellrc_path)
        .context(area_err!("Could not open shellrc_file"))?;*/
    dbg!(shellrc_path.to_str().unwrap());

    // Find if shell rc contains `aliases`
    let aliases_temp = path::absolute(&aliases_path)?;
    let aliases_path_str = aliases_temp.to_str().unwrap();
    // let aliases_reg = Regex::new(&format!(r"\\n(\\s*\\{})", aliases_path_str))?;
    let mut shellrc_file = File::options()
        .append(true)
        .create(true)
        .open(&shellrc_path)
        .context(area_err!("Shell rc file could not be opened"))?;
    let shellrc_contents = fs::read_to_string(&shellrc_path).unwrap();
    if !shellrc_contents.contains(aliases_path_str) {
        println!("`aliases` file not found in shell rc, inserting...");
        writeln!(
            shellrc_file,
            "{}",
            path::absolute(&aliases_path)?.to_str().unwrap()
        )?;
        dbg!(path::absolute(&aliases_path)?.to_str().unwrap());
    }

    let args = CarapaceArgs::parse();
     
    quit(&json_path, &json_val);
}

/// Formats an error message with its location (file, line and column)
/// Only use with `anyhow::context`
macro_rules! area_err {
    ($a:expr) => {
        format!("[{}:{}:{}] {}", file!(), line!(), column!(), $a)
    };
}
pub(crate) use area_err;

fn quit(path: &Path, value: &Value) -> ! {
    // Truncate the file and quit
    let file = File::create(path).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value).expect("Could not read Value");
    writer.flush().unwrap();
    std::process::exit(0);
}
