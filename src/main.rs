use std::error::Error;
use std::fs::{self, OpenOptions, File};
use std::io::{BufReader, BufWriter, Read, Write};
use serde_json::{Value};
use std::env;

/* "Result<(), Box<dyn Error>>" may return an error implementing
   Error. "dyn" ensures it is done at runtime. */
fn main() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = env::args().collect();
    let file: fs::File = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("settings.json")?;
    let reader = BufReader::new(&file);
    let writer = BufWriter::new(&file);
    
    let user_data: Value;

    match fs::exists("settings.json") {
        Ok(true) => user_data = serde_json::from_reader(reader)?,
        Ok(false) => todo!("Make an initializer helper function"),
        Err(e) => return Err(Box::new(e)),
    }
    dbg!(&user_data);
    println!("{}", user_data["name"]);
    Ok(())
}

fn find_shell_path<'a>(s: &'a str) {



}