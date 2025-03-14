use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum RowError {
    TableDoesntExist,
    FailedInsert,
    ReadMissingKey(String, String),
    MalformedID,
    FailedRead, // This error should not exist and is just stubbing actual file operation errors
    FailedToFindRecord,
}

impl Display for RowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_msg: String = match self {
            RowError::TableDoesntExist => "Tried to operate on a table that does not exist".to_string(),
            RowError::FailedInsert => "Failed insert row".to_string(),
            RowError::ReadMissingKey(key, key_type) => format!("Attempted to read record while missing '{}' {} field", key, key_type),
            RowError::MalformedID => "Provided ID was not valid".to_string(),
            RowError::FailedRead => "Failed to read data from the db (This error should not exist)".to_string(),
            RowError::FailedToFindRecord => "Failed to find a row with the given criteria".to_string(),
        };
        write!(f, "{}", err_msg)
    }
}

impl std::error::Error for RowError {}
