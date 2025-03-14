pub mod row_err;

use uuid::Uuid;

use serde_json::{Map, Value};
use crate::rows::row_err::RowError;
use crate::rows::row_err::RowError::{FailedInsert, TableDoesntExist};
use crate::State;
use crate::file_reader;

/*
    Rows are stored in sub_table files. A row has an ID that takes the form of `{usize}.{uuid}` where
    the first segment is a usize indicating which sub-table file a row is stored in, and the second
    segment is a UUID. If a row object in the 'foo' table is accessed by some ID that looks like
    `4.ABC-123-456` then the record will be in the /db_files/foo/sub_table_4.etch file. This schema
    works fine when working with objects by ID or without many concurrent requests but this does not
    scale or work if access is made by means other than ID
*/

fn generate_new_id(sub_table_index: usize) -> String {
    let id = Uuid::new_v4();
    format!("{}.{}", sub_table_index, id)
}

// TODO: The error handling of this file is abysmal

pub fn insert_data(state: &mut State, table_name: &str, mut data: Map<String, Value>) -> Result<String, RowError> {
    if !state.tables.contains_key(table_name) {
        return Err(TableDoesntExist)
    }

    // Get the index of the first sub_table which has space for a new record
    let mut table_metadata = file_reader::read_table_metadata(table_name).map_err(|_| FailedInsert)?;
    let mut sub_table_index: Option<usize> = None;
    for (index, value) in table_metadata.sub_tables.iter().enumerate() {
        if *value < table_metadata.records_per_sub_table {
            sub_table_index = Some(index);
        }
    }

    // Create a new sub_table if none of the existing ones have space
    if sub_table_index.is_none() {
        let new_index = table_metadata.sub_tables.len();
        // TODO: This is not ACID, if anything fails after the sub_table is updated with a 1 here then the db is in a bad state
        table_metadata.sub_tables.push(1);
        file_reader::replace_table_metadata(table_name, &table_metadata).map_err(|_| FailedInsert)?;
        file_reader::create_table_sub_table(table_name, new_index).map_err(|_| FailedInsert)?;
        sub_table_index = Some(new_index);
    }

    // Insert a record into the current sub_table
    let sub_table_index = sub_table_index.expect("Sub table index must be Some at this point");
    let id = generate_new_id(sub_table_index);
    data.insert("_id".to_string(), Value::String(id.clone()));
    let serialized = serde_json::to_string(&data).map_err(|_| FailedInsert)?;
    table_metadata.sub_tables[sub_table_index] += 1;
    file_reader::replace_table_metadata(table_name, &table_metadata).map_err(|_| FailedInsert)?;
    file_reader::insert_record_to_sub_table(table_name, sub_table_index, serialized).map_err(|_| FailedInsert)?;

    Ok(id)
}

pub fn read_data_by_id(_state: &State, table_name: &str, data: Map<String, Value>) -> Result<Value, RowError> {
    // Read which sub_table the record is in from the ID
    let target_id = match data.get("_id") {
        Some(Value::String(string_field)) => string_field,
        _ => return Err(RowError::ReadMissingKey("_id".to_string(), "string".to_string())),
    };
    let index_as_str = target_id.split(".").next().ok_or(RowError::MalformedID)?;
    let sub_table_index: usize = index_as_str.parse().map_err(|_| RowError::MalformedID)?;


    // TODO: De-serializing an entire file to search for a record seems pretty inefficient
    // TODO: This is really poor code, but it was written at 1am
    // Read the sub-table containing our record
    let sub_table_contents = file_reader::read_sub_table(table_name, sub_table_index).map_err(|_| RowError::FailedRead)?;
    match sub_table_contents {
        Value::Array(contents) => {
            for item in &contents {
                match item {
                    Value::Object(obj) => {
                        // This error is not meaningful and should return a malformed table file error instead
                        match obj.get("_id").ok_or(RowError::FailedRead)? {
                            Value::String(obj_key) => {
                                if obj_key == target_id {
                                    return Ok(item.clone())
                                }
                            },
                            // This error is not meaningful and should return a malformed table file error instead
                            _ => return Err(RowError::FailedRead)
                        }
                    },
                    // This error is not meaningful and should return a malformed table file error instead
                    _ => return Err(RowError::FailedRead)
                }
            }
        },
        // This error is not meaningful and should return a malformed table file error instead
        _ => return Err(RowError::FailedRead)
    }
    Err(RowError::FailedToFindRecord)
}
