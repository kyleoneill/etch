pub mod row_err;

use uuid::Uuid;

use serde_json::Value;
use crate::rows::row_err::RowError;
use crate::rows::row_err::RowError::{FailedInsert, TableDoesntExist};
use crate::State;
use crate::file_reader;

/*
    ID for a row object will take the form of NUM.NUM where the first num is the file it's in and the
    second is the ID within that file. When a file has some arbitrary number of objects in it, it will
    be considered "full" and we will move to the next file. This is probably a terrible way to store
    data :)
*/

fn generate_new_id(sub_table_index: usize) -> String {
    let id = Uuid::new_v4();
    format!("{}.{}", sub_table_index.to_string(), id.to_string())
}

pub fn insert_data(state: &mut State, table_name: &str, mut data: serde_json::Map<String, Value>) -> Result<String, RowError> {
    // TODO: The error handling of this function is abysmal

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
    table_metadata.sub_tables[sub_table_index] = table_metadata.sub_tables[sub_table_index] + 1;
    file_reader::replace_table_metadata(table_name, &table_metadata).map_err(|_| FailedInsert)?;
    file_reader::insert_record_to_sub_table(table_name, sub_table_index, serialized).map_err(|_| FailedInsert);

    Ok(id)
}
