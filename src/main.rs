use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};
use std::fs::{File};
use std::path::Path;
use std::io::{BufReader, BufWriter, Read, Write};

fn main() -> Result<()> {

    let file_path = Path::new("settings.json");
    let file: File;

    // Check if settings.json exists and create it if no
    if Path::try_exists(file_path)? {
        file = File::open(file_path)
            .context(format!("\"{}\" could not be opened.",
                file_path.file_name().unwrap().to_str().unwrap()))?;
    } else {
        file = File::create_new(file_path)?;
    }
    let reader = BufReader::new(&file);
    // Creates a mutable copy of settings.json
    let json_map: Map<String, Value> = if file.metadata()?.len() == 0 {
        serde_json::Map::new()
    } else {
        let json_temp: Value = serde_json::from_reader(reader)?;
        match json_temp {
            Value::Object(map) => map,
            _ => bail!("JSON root is not an object"),
        }
    };
    dbg!(json_map);
    Ok(())
}