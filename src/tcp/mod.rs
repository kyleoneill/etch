use std::fmt::{Display, Formatter};

pub mod connection;
pub mod frame;

#[derive(Debug)]
pub enum TCPError {
    //BufferOverflow,
    InvalidStart,
    MalformedJSON,
    MalformedPacket,
    FailedReadHeader,
    ParseFrame(String),
}

impl Display for TCPError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_msg: String = match self {
            //TCPError::BufferOverflow => "Internal buffer overflowed while reading packet".to_string(),
            TCPError::InvalidStart => "Received packet with invalid start byte".to_string(),
            TCPError::MalformedJSON => "Received packet with invalid JSON".to_string(),
            TCPError::MalformedPacket => "Received packet with a length that did not match header metadata".to_string(),
            TCPError::FailedReadHeader => "Failed to read the header of an incoming packet".to_string(),
            TCPError::ParseFrame(reason) => format!("Failed to parse a frame with reason: {}", reason),
        };
        write!(f, "{}", err_msg)
    }
}

impl std::error::Error for TCPError {}
