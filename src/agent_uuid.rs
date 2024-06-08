use std::fs::{write, File};
use std::io::prelude::*;

use crate::error::AgentError;

pub fn get_uuid() -> Result<String, AgentError> {
    let mut f = File::open("config/uuid")?;
    let mut buf = [0; 36];
    f.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

pub fn set_uuid(uuid: String) -> Result<(), AgentError> {
    // remove leading and trailing double quotes
    let mut uuid_chars = uuid.chars();
    uuid_chars.next();
    uuid_chars.next_back();

    write("config/uuid", uuid_chars.as_str())?;
    Ok(())
}
