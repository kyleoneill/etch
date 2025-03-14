pub mod table_err;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::tcp::frame::Frame;
use table_err::TableError;
use crate::State;
use crate::tables::table_err::TableError::{TableAlreadyExists};
use crate::file_reader;

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    name: String,
    field_type: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Constraint {
    field: String,
}

/// A database table, serialized into a JSON string for storage on disk.
#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    pub name: String,
    fields: Vec<Field>,
    constraints: Vec<Constraint>
}

impl Table {
    pub fn create_table(state: &mut State, frame: Frame) -> Result<(), TableError> {
        if state.tables.contains_key(frame.table.as_str()) {
            return Err(TableAlreadyExists)
        }

        // TODO: Actually create fields for the table
        let table = Self{ name: frame.table.clone(), fields: Vec::new(), constraints: Vec::new() };

        file_reader::create_new_table_file_data(&table)?;

        // Add new table to state
        state.tables.insert(table.name.clone(), table);
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableMetadata {
    pub records_per_sub_table: usize,
    pub sub_tables: Vec<usize>,
}
