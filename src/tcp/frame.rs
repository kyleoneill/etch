use serde_json::{Value, Map};
use super::TCPError;

#[derive(Debug)]
pub enum Command {
    Insert,
    Read,
    Update,
    Delete,
    CreateTable,
    DropTable,
}

impl Command {
    pub fn from_value(value: &Value) -> Result<Self, TCPError> {
        match value {
            Value::String(string) => match string.as_str() {
                "insert" => Ok(Self::Insert),
                "read" => Ok(Self::Read),
                "update" => Ok(Self::Update),
                "delete" => Ok(Self::Delete),
                "create_table" => Ok(Self::CreateTable),
                "drop_table" => Ok(Self::DropTable),
                _ => Err(TCPError::ParseFrame("Command was not a valid value".to_string())),
            },
            _ => Err(TCPError::ParseFrame("Command was not a string".to_string()))
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    pub command: Command,
    pub table: String,
    pub data: Map<String, Value>,
}

impl Frame {
    pub fn from_json(value: Value) -> Result<Self, TCPError> {
        match value {
            Value::Object(map) => {
                let command = Command::from_value(map.get("command").ok_or(TCPError::ParseFrame("Frame did not have a 'command' key".to_string()))?)?;
                let table = match map.get("table").ok_or(TCPError::ParseFrame("Frame did not have a 'table' key".to_string()))? {
                    Value::String(table_name) => table_name.to_owned(),
                    _ => return Err(TCPError::ParseFrame("Frame 'table' key was not a string".to_string()))
                };
                let data = match map.get("data").ok_or(TCPError::ParseFrame("Frame did not have a 'data' key".to_string()))? {
                    Value::Object(obj) => obj.to_owned(),
                    _ => return Err(TCPError::ParseFrame("Frame 'data' key was not an object".to_string()))
                };
                Ok(Self { command, table, data })
            },
            _ => Err(TCPError::ParseFrame("Frame's top level was not a dict object".to_string()))
        }
    }
}
