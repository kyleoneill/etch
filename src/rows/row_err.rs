use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum RowError {
    TableDoesntExist,
    FailedInsert,
}

impl Display for RowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_msg: String = match self {
            RowError::TableDoesntExist => "Tried to operate on a table that does not exist".to_string(),
            RowError::FailedInsert => "Failed insert row".to_string(),
        };
        write!(f, "{}", err_msg)
    }
}

impl std::error::Error for RowError {}
