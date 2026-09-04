mod args;

use anyhow::{Context, Result, bail};
use clap::Parser;
use serde_json::{Map, Value};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{self, Path, PathBuf};

use crate::args::{ActionType, AliasSubCommand, CarapaceArgs};

#[cfg(not(unix))]
compile_error!("MICROSLOP LOVER AHHHHHHHHHH");

fn main() -> Result<()> {
    let data_dir = directories::ProjectDirs::from("", "", "carapace-shortcutter")
        .context(format_err!(
            "Could not determine application data directory"
        ))?
        .data_dir()
        .to_owned();
    fs::create_dir_all(&data_dir).context(format_err!("Could not create directory(s)"))?;

    let json_path = data_dir.join("data.json");
    let aliases_path = data_dir.join("shortcuts");
    let json_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(json_path.to_str().unwrap())
        .context(format_err!("Could not open settings.json"))?;
    let mut shortcuts_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o755)
        .open(aliases_path.to_str().unwrap())
        .context(format_err!("Could not open `aliases` file"))?;
    let reader = BufReader::new(&json_file);

    // Either creates a mutable copy of settings.json or initializes an empty one
    let mut json_val = if json_file.metadata()?.len() == 0 {
        Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_reader(reader).context(format_err!("Invalid syntax"))?
    };

    let mut rc_path: PathBuf = PathBuf::new();
    let home_dir_string: String = directories::UserDirs::new()
        .unwrap()
        .home_dir()
        .to_string_lossy()
        .into_owned();

    let args = CarapaceArgs::parse();
    if matches!(args.command, ActionType::Setup) {
        setup(&home_dir_string, &mut rc_path, &mut json_val, &aliases_path)?;
        json_save(&json_path, &json_val)?;
        return Ok(());
    }

    let settings_json = json_val
        .as_object_mut()
        .context(format_err!("Not an object"))?;

    // Asks for the shell's rc file if it isn't found
    if settings_json.contains_key("rc_file") {
        rc_path = PathBuf::from(
            settings_json["rc_file"]
                .as_str()
                .context(format_err!("Error reading JSON"))?
                .replace('~', &home_dir_string),
        );
    } else {
        bail!("ERROR: The shell's rc file was not specified, run `csc setup`");
    }

    match args.command {
        ActionType::Alias(alias_command) => {
            let aliases = settings_json["aliases"]
                .as_object_mut()
                .context(format_err!("Internal JSON error, run `csc` setup"))?;
            match alias_command.subcommand {
                AliasSubCommand::Create(create_request) => {
                    writeln!(
                        shortcuts_file,
                        "alias {}={}",
                        create_request.trigger, create_request.aliasee
                    )?; // Write the alias command to shortcuts_file and to data.json
                    println!(
                        "Adding alias '{} -> {}'...",
                        create_request.trigger, create_request.aliasee
                    );
                    if let Some(old_alias) = aliases.insert(
                        create_request.trigger,
                        Value::String(create_request.aliasee),
                    ) { // Check if an alias already exists
                        
                        println!("Replaced old alias '{}'", old_alias.as_str().unwrap_or("Invalid"));
                    }
                }
                AliasSubCommand::Remove(remove_request) => {
                    if let Some(old_alias) = aliases.remove(&remove_request.shortcut) {
                        println!(
                            "Deleted '{} -> {}'",
                            remove_request.shortcut, old_alias
                        ); // Print the trigger and the aliasee
                    } else {
                        eprintln!("ERROR: alias '{}' never existed.", remove_request.shortcut);
                    }
                }
            }
        }
        ActionType::Moniker(_x) => {
            todo!("Make moniker functionality");
        }

        ActionType::Synchronize => {
            todo!("Make synchronizer functionality");
        }

        ActionType::Setup => bail!(
            "ERROR: `ObjectType::Setup` was detected after \
                setup sequence, this should never happen"
        ),
    }

    json_save(&json_path, &json_val)?;
    Ok(())
}

/// Formats an error message with its location (file, line and column)
///
/// Only use with `anyhow::context`
macro_rules! format_err {
    ($a:expr) => {
        format!("\"{}\" at {}:{}:{}", $a, file!(), line!(), column!())
    };
}
pub(crate) use format_err;

fn setup(
    home_dir_string: &str,
    rc_path: &mut PathBuf,
    json_val: &mut Value,
    aliases_path: &Path,
) -> Result<()> {
    let mut stdin_buf = String::new();
    print!(
        "Please provide the RELATIVE path to the user's \
        shell rc file (.zshrc, .bashrc), \
        or leave a newline to find it automatically.\n\n\
        ~/"
    );
    loop {
        io::stdout().flush()?;
        io::stdin().read_line(&mut stdin_buf)?;
        if stdin_buf.trim().is_empty() {
            let shell_rc_buf = find_shell_rc(home_dir_string);
            if let Err(err) = shell_rc_buf {
                eprint!(
                    "ERROR: {}\nA shell rc file could not be found. \
                    Please manually input the path to one.\n\
                    ~/",
                    err
                );
                continue;
            }
            stdin_buf = unsafe { shell_rc_buf.unwrap_unchecked() };
        }
        break;
    }
    dbg!(&stdin_buf);
    rc_path.push(format!("{home_dir_string}/{}", stdin_buf.trim()));
    let settings_json = json_val
        .as_object_mut()
        .context(format_err!("Not an object"))?;

    // Initialize settings.json
    settings_json.insert(
        "rc_file".to_owned(),
        Value::String(
            rc_path
                .to_string_lossy()
                .into_owned()
                .replace(home_dir_string, "~"),
        ),
    );
    settings_json.insert("aliases".to_owned(), Value::Object(Map::new()));
    settings_json.insert("monikers".to_owned(), Value::Object(Map::new()));

    let aliases_path_str = aliases_path.to_str().unwrap();
    let rc_contents = fs::read_to_string(&rc_path)?;

    let mut rc_file = File::options()
        .append(true)
        .create(true)
        .open(&rc_path)
        .context(format_err!(
            "Shell rc file could not be opened, check the user's permissions"
        ))?;

    if !rc_contents.contains(aliases_path_str) {
        println!("`aliases` file not found in shell rc, inserting...");
        writeln!(
            rc_file,
            ". {} # Generated by carapace-shortcutter",
            path::absolute(aliases_path)?
                .to_str()
                .unwrap()
                .replace(home_dir_string, "~") // Make the path relative
                .replace(' ', "\\ ") //           and escape any spaces
        )?;
    }
    Ok(())
}

/// Finds the shell's rc depending on the current shell
fn find_shell_rc(home_string: &str) -> Result<String> {
    let current_shell = env::var("SHELL").context(format_err!(
        "SHELL environment variable not found. Please set it to the path of your current shell."
    ))?;

    // The path relative to the home directory (.bashrc, .zshrc)
    let mut relative_rc_path = String::new();

    if current_shell.contains("/bash") {
        ".bashrc".clone_into(&mut relative_rc_path);
    } else if current_shell.contains("fish") {
        ".config/fish/config.fish".clone_into(&mut relative_rc_path);
    } else if current_shell.contains("/zsh") {
        const ZSH_RC_PRIORITY: [&str; 3] = [".zshrc", ".config/.zshrc", ".config/zsh/.zshrc"];

        // Loop through the priority list, stopping once an existing file has been found
        for zsh_relative_rc_path in ZSH_RC_PRIORITY {
            let zsh_absolute_rc_path = format!("{home_string}/{zsh_relative_rc_path}");

            if fs::exists(Path::new(&zsh_absolute_rc_path)).unwrap_or(false) {
                zsh_relative_rc_path.clone_into(&mut relative_rc_path);
                break;
            }
        }
    }

    if relative_rc_path.is_empty() {
        bail!("Your shell is not currently supported");
    }
    println!("Choosing {home_string}/{relative_rc_path} as shell rc...");
    Ok(relative_rc_path.clone())
}

fn json_save(path: &Path, value: &Value) -> Result<()> {
    // Truncate the file and overwrite it
    let mut file = File::create(path).unwrap();
    //let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut file, value).context(format_err!("Could not read Value"))?;
    file.flush().unwrap();
    Ok(())
}
