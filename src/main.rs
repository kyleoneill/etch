mod tcp;
mod tables;
mod rows;
mod file_reader;

use std::collections::HashMap;
use serde_json::json;
use tables::Table;

use tokio::net::{TcpListener, TcpStream};
use crate::tcp::connection::Connection;
use crate::tcp::frame::Command;

#[derive(Debug)]
pub struct State {
    tables: HashMap<String, Table>,
}

impl State {
    fn initialize() -> Self {
        let tables = match file_reader::load_tables_from_disk() {
            Ok(tables) => tables,
            Err(e) => panic!("Failed to load tables with error: {}", e)
        };
        Self{ tables }
    }
}

#[tokio::main]
async fn main() {
    // Bind a listener for TCP requests
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("Failed to bind a TCP listener");

    file_reader::check_for_db_dir();

    // Load db state
    let mut state = State::initialize();

    // Loop and listen for connection requests
    loop {
        // TODO: Should print or log rather than panic
        let (stream, _address) = match listener.accept().await {
            Ok(res) => res,
            Err(e) => panic!("Failed to accept a connection with error: {:?}", e)
        };
        // TODO: Respond to the client?
        //       process should return some sort of UserResponse struct maybe,
        //       or just something that impls serialize so unique responses from each
        //       command can be wrapped into a response obj
        //
        // Responses should have a code, like "failed", "created", "deleted", etc
        process(&mut state, stream).await;
    }
}

async fn process(state: &mut State, stream: TcpStream) {
    let mut connection = Connection::new(stream);
    match connection.read_frame().await {
        Ok(frame) => {
            // TODO: Response should be an actual struct and constructed better
            let res_data = match frame.command {
                Command::Insert => {
                    match rows::insert_data(state, frame.table.as_str(), frame.data) {
                        Ok(id) => {
                            json!({
                                "code": 201,
                                "data": {
                                    "id": id
                                }
                            })
                        }, // TODO: Return the ID to the user
                        Err(e) => {
                            eprintln!("Error while processing insert row command: {}", e);
                            json!({
                                "code": 500,
                                "data": {
                                    "msg": "Error while processing insert row"
                                }
                            })
                        }
                    }
                },
                Command::Read => todo!("Read command"),
                Command::Update => todo!("Update command"),
                Command::Delete => todo!("Delete command"),
                Command::CreateTable => {
                    match Table::create_table(state, frame) {
                        Ok(()) => json!({
                            "code": 201,
                            "data": {}
                        }),
                        Err(e) => {
                            eprintln!("Error while processing create table command: {}", e);
                            json!({
                                "code": 500,
                                "data": {
                                    "msg": "Error while creating table"
                                }
                            })
                        }
                    }
                },
                Command::DropTable => todo!("DropTable command"),
            };
            match connection.respond(res_data).await {
                Ok(written_bytes) => println!("Responded to request with {} bytes", written_bytes),
                Err(e) => eprintln!("Failed to respond to requester with error: {}", e)
            }
        },
        Err(e) => eprintln!("Failed to read frame with error: {}", e)
    }
}
