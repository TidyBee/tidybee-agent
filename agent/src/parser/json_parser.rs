use std::fs::File;
use file_into_string::*;
use json;
use json::JsonValue;
use std::error::Error;

pub fn read_value_from_file(path: &str, value: String) -> Result<JsonValue, Box<dyn Error>>{
    let file = File::open(path).unwrap();
    let string = file_into_string(file).unwrap();
    let parsed = json::parse(&string).unwrap();

    Ok(parsed[value].clone())
}