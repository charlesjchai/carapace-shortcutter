#[cfg(all(not(unix), not(windows)))]
compile_error!(
    "Your operating system is not a Unix-based operating system, only Unix-based operating systems are supported."
);

#[cfg(windows)]
compile_error!("MICROSLOP LOVER AHHHHHHHHHH");

mod args;

use anyhow::{Context, Result, bail};
use clap::Parser;
use directories::{ProjectDirs, UserDirs};
use mlua::Lua;
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Seek, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::args::{ActionType, AliasSubCommand, CarapaceArgs, MonikerCommand, MonikerSubCommand};

static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    ProjectDirs::from("", "", "carapace-shortcutter")
        .expect("Could not determine application data directory")
        .data_dir()
        .to_owned()
});
static HOME_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| UserDirs::new().unwrap().home_dir().to_path_buf());
static MONIKER_DIR: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("monikers"));

fn main() -> Result<()> {
    let json_path = DATA_DIR.join("data.json");
    let aliases_path = DATA_DIR.join("shortcuts");
    // Create data_dir and moniker_dir in one fell swoop
    fs::create_dir_all(&*MONIKER_DIR).context(area_err!("Could not create directory(s)"))?;

    let mut json_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(json_path.to_str().unwrap())
        .context(area_err!("Could not open settings.json"))?;
    let mut shortcuts_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o755)
        .open(aliases_path.to_str().unwrap())
        .context(area_err!("Could not open `aliases` file"))?;
    let reader = BufReader::new(&json_file);

    // Either creates a mutable copy of settings.json or initializes an empty one
    let mut json_val = if json_file.metadata()?.len() == 0 {
        Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_reader(reader).context(area_err!("Invalid syntax"))?
    };

    let mut rc_path: PathBuf = PathBuf::new();

    let args = CarapaceArgs::parse();
    if let ActionType::Moniker(MonikerCommand {
        subcommand: MonikerSubCommand::Execute(execute_request),
    }) = args.command
    {
        // Handle the `csc moniker execute` command
        let moniker_name = execute_request.moniker;
        let moniker_name_lua = format!("{moniker_name}.lua");

        let moniker_code = fs::read_to_string(MONIKER_DIR.join(&moniker_name_lua)).context(
            area_err!(format!("Moniker file `{moniker_name_lua}` not found")),
        )?;

        let lua = Lua::new();
        let arg_table = lua.create_table().expect("Couldn't create table");
        for (i, argument) in execute_request.args.into_iter().enumerate() {
            // Lua arrays are 1-indexed
            arg_table.set(i + 1, argument).unwrap();
        }
        lua.globals()
            .set("arg", arg_table)
            .expect("Couldn't insert table");
        let status: Option<i32> = lua
            .load(&moniker_code)
            .set_name(&moniker_name_lua)
            .eval()
            .unwrap_or_else(|e| panic!("Moniker `{moniker_name_lua}` failed with error: {e:?}"));
        if status.unwrap_or(0) != 0 {
            bail!(
                "Moniker {moniker_name_lua} returned non-zero return value: {}",
                status.unwrap()
            );
        }
        return Ok(());
    } else if matches!(args.command, ActionType::Setup) {
        setup(&mut rc_path, &mut json_val, &aliases_path)?;
        json_synchronize(&mut json_file, &mut shortcuts_file, &json_val)?;
        return Ok(());
    } else if !["rc_file", "aliases", "monikers"].iter().any(|k| {
        json_val
            .as_object()
            .expect("Not an object")
            .contains_key(k.to_owned())
    }) {
        eprintln!("Necessary keys not found in JSON, running setup...");
        setup(&mut rc_path, &mut json_val, &aliases_path)?;
        json_synchronize(&mut json_file, &mut shortcuts_file, &json_val)?;
        //return Ok(());
    }

    let settings_json = json_val
        .as_object_mut()
        .context(area_err!("Not an object"))?;

    parse_args(args, settings_json, &aliases_path)?;

    json_synchronize(&mut json_file, &mut shortcuts_file, &json_val)?;
    Ok(())
}

/// Formats an error message with its location (file, line and column)
/// Only use with `anyhow::context`
/// ```
/// let none = None;
/// let val = none.context(area_err!("None detected"))?;
/// // Returns "Error: "None detected" at file:line:column "
/// ```
macro_rules! area_err {
    ($a:expr) => {
        format!("\"{}\" at {}:{}:{}", $a, file!(), line!(), column!())
    };
}
pub(crate) use area_err;

trait ShellString {
    fn shell_escape_chars(&self) -> Cow<'_, str>;
    fn path_to_relative(&self) -> Cow<'_, str>;
}
impl ShellString for str {
    /// This function escapes any special characters and prepares the string for a shell
    fn shell_escape_chars(&self) -> Cow<'_, str> {
        const ESCAPE_CHARACTERS: &str = r#"\$ `"<>|;&()*?[]!{}~#"#;
        let mut result = String::with_capacity(self.len());
        let mut is_modified = false;

        for c in self.chars() {
            if ESCAPE_CHARACTERS.contains(c) {
                result.push('\\');
                is_modified = true;
            }
            result.push(c);
        }
        if is_modified {
            Cow::Owned(result)
        } else {
            Cow::Borrowed(self)
        }
    }
    /// Converts an absolute path (in the form of a &str) to a relative path
    fn path_to_relative(&self) -> Cow<'_, str> {
        let home_dir_string = HOME_DIR.to_str().unwrap();
        if self.contains(home_dir_string) {
            let relative_path = self.replace(home_dir_string, "~");
            Cow::Owned(relative_path)
        } else {
            Cow::Borrowed(self)
        }
    }
}
fn setup(rc_path: &mut PathBuf, json_val: &mut Value, aliases_path: &Path) -> Result<()> {
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
            let shell_rc_buf = find_shell_rc();
            if let Err(err) = shell_rc_buf {
                eprint!(
                    "ERROR: {err}\nA shell rc file could not be found. \
                    Please manually input the path to one.\n\
                    ~/"
                );
                continue;
            }
            stdin_buf = unsafe { shell_rc_buf.unwrap_unchecked() };
        }
        break;
    }

    rc_path.push(HOME_DIR.join(stdin_buf.trim()));
    let settings_json = json_val
        .as_object_mut()
        .context(area_err!("Not an object"))?;

    // Initialize settings.json
    settings_json.insert(
        "rc_file".to_string(),
        Value::String(rc_path.to_str().unwrap().path_to_relative().into_owned()),
    );
    settings_json
        .entry("aliases".to_string())
        .or_insert(Value::Object(Map::new()));
    settings_json
        .entry("monikers".to_string())
        .or_insert(Value::Array(Vec::new()));

    let aliases_path_str = aliases_path.to_str().unwrap();
    let rc_contents = fs::read_to_string(&rc_path)?;

    let mut rc_file = File::options()
        .append(true)
        .create(true)
        .open(&rc_path)
        .context(area_err!(
            "Shell rc file could not be opened, check the user's permissions"
        ))?;

    if !rc_contents.contains(
        aliases_path_str
            .shell_escape_chars()
            .path_to_relative()
            .as_ref(),
    ) {
        println!("`aliases` file not found in shell rc, inserting...");
        writeln!(
            rc_file,
            ". {} # Generated by carapace-shortcutter",
            fs::canonicalize(aliases_path)
                .expect("File doesn't exist.")
                .to_str()
                .unwrap()
                .shell_escape_chars()
                .path_to_relative()
        )?;
    }
    Ok(())
}

/// Finds the shell's rc depending on the current shell
fn find_shell_rc() -> Result<String> {
    let current_shell = env::var("SHELL").context(area_err!(
        "$SHELL environment variable not found. Please set it to the path of your current shell."
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
            let zshrc_path = HOME_DIR.join(zsh_relative_rc_path);
            let zshrc_path_str = zshrc_path.to_str().unwrap();

            if fs::exists(Path::new(&zshrc_path_str)).unwrap_or(false) {
                zsh_relative_rc_path.clone_into(&mut relative_rc_path);
                break;
            }
        }
    }

    if relative_rc_path.is_empty() {
        bail!("Your shell is not currently supported");
    }
    println!(
        "Choosing {} as shell rc...",
        HOME_DIR.join(&relative_rc_path).to_str().unwrap()
    );
    Ok(relative_rc_path)
}

/// Parses the args given to the function
fn parse_args(
    args: CarapaceArgs,
    settings_json: &mut Map<String, Value>,
    aliases_path: &Path,
) -> Result<()> {
    match args.command {
        ActionType::Alias(alias_command) => {
            let aliases = settings_json
                .get_mut("aliases")
                .context(area_err!("`aliases` key not found, run `csc setup`"))?
                .as_object_mut()
                .context(area_err!("Not an object"))?;
            // Remove aliases that are not a string
            aliases.retain(|_k, value| value.is_string());
            match alias_command.subcommand {
                AliasSubCommand::Add(create_request) => {
                    println!(
                        "Adding alias '{}' -> '{}'...",
                        create_request.alias, create_request.old_command
                    );
                    if let Some(old_alias) = aliases.insert(
                        create_request.alias,
                        Value::String(create_request.old_command),
                    ) {
                        // Check if an alias already exists
                        println!("Replaced old alias '{old_alias}'");
                    }
                    println!(
                        "Done, restart your terminal or run `source {}` for changes to take affect.",
                        aliases_path
                            .to_str()
                            .unwrap()
                            .shell_escape_chars()
                            .path_to_relative()
                    );
                }
                AliasSubCommand::Del(remove_request) => {
                    if let Some(old_alias) = aliases.remove(&remove_request.alias) {
                        println!("Deleted '{}' -> '{}'", remove_request.alias, old_alias); // Print the trigger and the aliasee
                    } else {
                        eprintln!("ERROR: alias '{}' never existed.", remove_request.alias);
                    }
                }
                AliasSubCommand::List => {
                    println!("Aliases:");
                    if aliases.is_empty() {
                        println!("(None)");
                    } else {
                        for (alias, old_command) in aliases {
                            println!("'{alias}' -> '{}'", old_command.as_str().unwrap());
                        }
                    }
                }
            }
        }
        ActionType::Moniker(moniker_command) => {
            let monikers = settings_json
                .get_mut("monikers")
                .context(area_err!("`monikers` key not found, run `csc setup`"))?
                .as_array_mut()
                .context(area_err!("Not an array"))?;
            // Remove monikers that are not a string and duplicates
            monikers.retain(Value::is_string);
            let mut seen = HashSet::new();
            monikers.retain(|item| seen.insert(item.clone()));
            match moniker_command.subcommand {
                MonikerSubCommand::Create(create_request) => {
                    if create_request
                        .moniker_path
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap()
                        != "lua"
                    {
                        bail!("You must input a .lua file.");
                    }
                    println!(
                        "Adding moniker '{}' -> '{}'...",
                        create_request.moniker,
                        create_request.moniker_path.to_str().unwrap()
                    );
                    let lua_file_path = fs::canonicalize(&create_request.moniker_path)
                        .context(area_err!("File does not exist"))?;
                    fs::hard_link(
                        &lua_file_path,
                        MONIKER_DIR.join(format!("{}.lua", create_request.moniker)),
                    )
                    .unwrap_or_else(|e| {
                        eprintln!("ERROR: Hard link failed: '{e:?}', falling back to a copy");
                        fs::remove_file(
                            MONIKER_DIR.join(format!("{}.lua", create_request.moniker)),
                        )
                        .expect("Failed to remove file");
                        fs::copy(
                            lua_file_path,
                            MONIKER_DIR.join(format!("{}.lua", create_request.moniker)),
                        )
                        .expect("Copy failed");
                    });
                    if monikers.contains(&Value::String(create_request.moniker.clone())) {
                        let old_moniker = monikers
                            .iter()
                            .position(|x| *x == *create_request.moniker)
                            .context(area_err!("Moniker does not exist"))?;
                        println!(
                            "Replacing old moniker '{} -> '{}'...",
                            monikers.get(old_moniker).unwrap(),
                            MONIKER_DIR
                                .join(
                                    monikers
                                        .get(old_moniker)
                                        .context(area_err!("Old moniker not found"))?
                                        .as_str()
                                        .context(area_err!("Not a string"))?
                                )
                                .to_str()
                                .unwrap()
                        );
                    }
                    monikers.push(Value::String(create_request.moniker));
                    println!(
                        "Done, restart your terminal or run `source {}` for changes to take affect.",
                        aliases_path
                            .to_str()
                            .unwrap()
                            .shell_escape_chars()
                            .path_to_relative()
                    );
                }
                MonikerSubCommand::Remove(remove_request) => {
                    if monikers.contains(&Value::String(remove_request.moniker.clone())) {
                        fs::remove_file(
                            MONIKER_DIR
                                .join(&remove_request.moniker)
                                .with_extension("lua"),
                        )
                        .context(area_err!("File removal failed"))?;
                        monikers.retain(|other_moniker| *other_moniker != remove_request.moniker);
                        println!(
                            "Deleted '{}' -> '{}'",
                            remove_request.moniker,
                            MONIKER_DIR
                                .join(&remove_request.moniker)
                                .to_str()
                                .unwrap()
                                .path_to_relative()
                        );
                    } else {
                        eprintln!("ERROR: moniker '{}' never existed.", remove_request.moniker);
                    }
                }
                MonikerSubCommand::List => {
                    println!("Monikers:");
                    if monikers.is_empty() {
                        println!("(None)");
                    } else {
                        for moniker in monikers {
                            let moniker_name =
                                moniker.as_str().context(area_err!("Not a string"))?;
                            println!(
                                "'{}' -> '{}'",
                                moniker_name,
                                MONIKER_DIR
                                    .join(format!("{moniker_name}.lua"))
                                    .to_str()
                                    .unwrap()
                                    .path_to_relative()
                            );
                        }
                    }
                }
                MonikerSubCommand::Execute(_) => bail!(
                    "ERROR: `MonikerSubCommand::Execute` was detected after \
                parsing, this should never happen"
                ),
            }
        }

        ActionType::Clean => todo!("Make clean command"),

        ActionType::Setup => bail!(
            "ERROR: `ActionType::Setup` was detected after \
                setup sequence, this should never happen"
        ),
    }
    Ok(())
}

/// Synchronizes the shortcuts with the JSON file and the JSON file with the object
fn json_synchronize(json_file: &mut File, shortcuts_file: &mut File, value: &Value) -> Result<()> {
    json_file.set_len(0)?;
    json_file.seek(io::SeekFrom::Start(0))?;

    serde_json::to_writer_pretty(&mut *json_file, value)
        .context(area_err!("Could not read Value"))?;
    json_file.flush().unwrap();
    let aliases = value
        .as_object()
        .context(area_err!("Not an object"))?
        .get("aliases")
        .context(area_err!("`aliases` key not found, run `csc setup`"))?
        .as_object()
        .context(area_err!("Not an object"))?;
    let monikers = value
        .as_object()
        .context(area_err!("Not an object"))?
        .get("monikers")
        .context(area_err!("`monikers` key not found, run `csc setup`"))?
        .as_array()
        .context(area_err!("Not an array"))?;

    // TODO: Move this to `clean` subcommand
    for entry in fs::read_dir(&*MONIKER_DIR)? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.is_dir() {
            fs::remove_dir(&file_path)?;
            continue;
        }
        if file_path.is_file() {
            let file_name = file_path.to_str().unwrap();
            if !monikers.contains(&Value::String(
                file_path
                    .with_extension("")
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            )) {
                fs::remove_file(&file_path)?;
                println!("Removed: {file_name}");
            }
        }
    }

    shortcuts_file.set_len(0)?;
    shortcuts_file.seek(io::SeekFrom::Start(0))?;

    let mut writer = BufWriter::new(shortcuts_file);
    for (alias, old_command) in aliases {
        writer.write_all(&format!("alias {alias}={old_command}\n").into_bytes())?;
    }
    for moniker in monikers {
        writer.write_all(
            &format!(
                "alias {0}='csc moniker execute {0}'",
                moniker.as_str().context(area_err!("Not a string"))?
            )
            .into_bytes(),
        )?;
    }
    writer.flush()?;
    Ok(())
}
