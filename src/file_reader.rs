use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use serde_json::{json, Value};
use crate::tables::table_err::TableError;
use crate::tables::{Table, TableMetadata};
use crate::tables::table_err::TableError::{FailedCreateDir, FailedDiskRead, FailedDiskWrite, FailedOpenTableFile};

// TODO: This should be an env var probably
const TABLE_FILE_NAME: &str = "tables.etch";

// TODO: This API is a bit of a mess and should be cleaned up

pub fn get_path_for_files() -> PathBuf {
    let mut current_dir = std::env::current_dir().expect("Failed to get current dir");
    current_dir.push("db_files");
    current_dir
}

pub fn check_for_db_dir() {
    let db_dir = get_path_for_files();
    if !fs::exists(&db_dir).expect("Failed to read db_files path to check if it exists") {
        fs::create_dir(db_dir).expect("Failed to create missing db_files directory")
    }
}


// TABLES

pub fn get_table_file_path() -> PathBuf {
    let mut table_file_dir = get_path_for_files();
    table_file_dir.push(TABLE_FILE_NAME);
    table_file_dir
}

fn create_file_with_empty_list(file_name: &Path) -> Result<usize, TableError> {
    let mut file = File::create(file_name).map_err(|_| FailedDiskWrite)?;
    let empty_list = Value::Array(Vec::new());
    let serialized = serde_json::to_string(&empty_list).expect("serde_json Value should impl Serialize");
    file.write(serialized.as_bytes()).map_err(|_| FailedDiskWrite)
}

/// Create a new table file, initialized to hold an empty JSON list.
pub fn create_table_file() -> Result<usize, TableError> {
    let table_file_path = get_table_file_path();
    create_file_with_empty_list(table_file_path.as_path())
}

pub fn open_table_file_read() -> Result<File, TableError> {
    let table_file_path = get_table_file_path();
    match File::open(&table_file_path) {
        Ok(file) => Ok(file),
        Err(_e) => {
            // Should actually handle the error, but we will assume that we are error-ing
            // because the file does not exist
            create_table_file()?;
            File::open(table_file_path).map_err(|_| FailedOpenTableFile)
        }
    }
}

pub fn open_table_file_write() -> Result<File, TableError> {
    let table_file_path = get_table_file_path();
    match OpenOptions::new().read(true).write(true).open(&table_file_path) {
        Ok(file) => Ok(file),
        Err(_e) => {
            create_table_file()?;
            OpenOptions::new().read(true).write(true).open(table_file_path).map_err(|_| FailedOpenTableFile)
        }
    }
}

/// Write a new table to disk, appending it to the end of the table file.
pub fn write_table_file_to_disk(table: &Table) -> Result<(), TableError> {
    let mut file = open_table_file_write()?;
    let serialized = serde_json::to_string(table).map_err(|_| FailedDiskWrite)?;
    file.seek(SeekFrom::End(-1)).expect("End of table file should always be more than 1 char away from the start");
    let res = match file.metadata().expect("Failed to get file metadata").len() {
        2 => write!(file, "{}]", serialized),
        _ => write!(file, ", {}]", serialized)
    };
    res.map_err(|_| FailedDiskWrite)
}

fn create_table_metadata(table_name: &str) -> Result<(), TableError> {
    let mut new_table_path = get_path_for_files();
    new_table_path.push(table_name);
    fs::create_dir(new_table_path.as_path()).map_err(|_| FailedCreateDir)?;
    new_table_path.push("metadata.etch");
    let data = json!({
        "sub_tables": [0],
        "records_per_sub_table": 1000,
    });
    let serialized = serde_json::to_string(&data).expect("serde_json Value should impl Serialize");
    fs::write(new_table_path, serialized).map_err(|_| FailedDiskWrite)
}

pub fn replace_table_metadata(table_name: &str, metadata: &TableMetadata) -> Result<(), TableError> {
    let mut new_table_path = get_path_for_files();
    new_table_path.push(table_name);
    new_table_path.push("metadata.etch");
    let serialized = serde_json::to_string(metadata).expect("serde_json Value should impl Serialize");
    fs::write(new_table_path, serialized).map_err(|_| FailedDiskWrite)
}

pub fn create_table_sub_table(table_name: &str, num: usize) -> Result<(), TableError> {
    let mut new_table_path = get_path_for_files();
    new_table_path.push(table_name);
    let sub_table_name = format!("sub_table_{}.etch", num);
    new_table_path.push(sub_table_name.as_str());
    let _res = create_file_with_empty_list(new_table_path.as_path())?;
    Ok(())
}

pub fn create_new_table_file_data(table: &Table) -> Result<(), TableError> {
    // TODO: If one op here fails the already finished ones should be rolled back?
    write_table_file_to_disk(table)?;
    create_table_metadata(table.name.as_str())?;
    create_table_sub_table(table.name.as_str(), 0)
}

pub fn load_tables_from_disk() -> Result<HashMap<String, Table>, TableError> {
    let mut table_file = open_table_file_read()?;
    let mut data = vec![];
    table_file.read_to_end(&mut data).map_err(|_| FailedDiskRead)?;
    let serialized_tables: Vec<Table> = serde_json::from_slice(&data).expect("Table file is corrupt and contents cannot be deserialized");
    let mut map: HashMap<String, Table> = HashMap::new();
    for table in serialized_tables {
        map.insert(table.name.clone(), table);
    }
    Ok(map)
}

pub fn read_table_metadata(table_name: &str) -> Result<TableMetadata, TableError> {
    let mut new_table_path = get_path_for_files();
    new_table_path.push(table_name);
    new_table_path.push("metadata.etch");
    let file_contents = fs::read(new_table_path.as_path()).map_err(|_| FailedDiskRead)?;
    serde_json::from_slice(&file_contents).map_err(|_| FailedDiskRead)
}

pub fn insert_record_to_sub_table(table_name: &str, sub_table_index: usize, record: String) -> Result<(), TableError> {
    let mut sub_table_path = get_path_for_files();
    sub_table_path.push(table_name);
    let sub_table_name = format!("sub_table_{}.etch", sub_table_index);
    sub_table_path.push(sub_table_name.as_str());

    let mut sub_table_file = OpenOptions::new().read(true).write(true).open(&sub_table_path).map_err(|_| FailedDiskRead)?;
    sub_table_file.seek(SeekFrom::End(-1)).expect("End of table file should always be more than 1 char away from the start");

    let res = match sub_table_file.metadata().expect("Failed to get file metadata").len() {
        2 => write!(sub_table_file, "{}]", record),
        _ => write!(sub_table_file, ", {}]", record)
    };
    res.map_err(|_| FailedDiskWrite)
}

pub fn read_sub_table(table_name: &str, sub_table_index: usize) -> Result<Value, TableError> {
    let mut sub_table_path = get_path_for_files();
    sub_table_path.push(table_name);
    let sub_table_name = format!("sub_table_{}.etch", sub_table_index);
    sub_table_path.push(sub_table_name.as_str());

    let file = fs::read(sub_table_path).map_err(|_| FailedDiskRead)?;
    serde_json::from_slice(&file).map_err(|_| FailedDiskRead)
}
