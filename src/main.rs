use anyhow::{Context, Result};
use std::fs::{File};
use std::path::Path;
use std::io::{BufReader, BufWriter, Read, Write};
use std::env;

fn main() -> Result<()> {

    let file_path = Path::new("settings.json");
    let file: File;

    // Check if settings.json exists and create it if no
    if Path::try_exists(file_path)? {
        file = File::open(file_path)?;
    } else {
        file = File::create_new(file_path)?;
    }
    let reader = BufReader::new(&file);
    let mut str_json: String = serde_json::from_reader(reader)
        .context("Failed to read json")?;
    if file.metadata()?.len() == 0 {
        str_json.push_str("{}");
    }

    Ok(())
}