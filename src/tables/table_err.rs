use std::fmt::{Display, Formatter};

// TODO: These are only here because of a refactor, file operation errors should really not be a part of
//       the table module

#[derive(Debug)]
pub enum TableError {
    FailedOpenTableFile,
    FailedDiskRead,
    FailedDiskWrite,
    TableAlreadyExists,
    FailedCreateDir,
}

impl Display for TableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_msg: String = match self {
            TableError::FailedOpenTableFile => "Failed to read or create the table file".to_string(),
            TableError::FailedDiskWrite => "Failed to write table data to disk".to_string(),
            TableError::FailedDiskRead => "Failed to read tables from disk".to_string(),
            TableError::TableAlreadyExists => "Tried to create a table which already exists".to_string(),
            TableError::FailedCreateDir => "Failed to create a directory for table".to_string(),
        };
        write!(f, "{}", err_msg)
    }
}

impl std::error::Error for TableError {}
