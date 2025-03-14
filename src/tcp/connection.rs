use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use super::TCPError;
use serde_json::Value;
use super::frame::Frame;

#[derive(Debug)]
pub struct Connection {
    // TcpStream is decorated with a BufWriter, which provides write level buffering
    stream: TcpStream,//BufWriter<TcpStream>,

    // TODO: Use this buffer instead of the two allocated ones in read_frame
    // Buffer for reading frames
    //buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            stream: socket,//BufWriter::new(socket),
            // 64KB is probably fine
            //buffer: BytesMut::with_capacity(64 * 1024),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Frame, TCPError> {
        // TODO: This method is very fragile and reading a packet here will cause a panic if
        //       all data is not transmitted at once, will need to be re-done but it's good
        //       enough for a hackathon

        // First 3 bytes are packet metadata
        let mut buffer = [0u8; 3];
        match self.stream.read_exact(&mut buffer).await {
            Ok(_size) => (),
            Err(_e) => return Err(TCPError::FailedReadHeader)
        }

        // First byte indicates start of transmission
        // Bytes 2 and 3 are a u16 indicating the length of the data to follow
        let start_byte = buffer[0];
        let data_length = u16::from_be_bytes([buffer[1], buffer[2]]) as usize;

        // The first byte being 42 indicates start of transmission
        if start_byte != 42 {
            return Err(TCPError::InvalidStart)
        }

        let mut data_buffer = vec![0u8; data_length];
        match self.stream.read_exact(&mut data_buffer).await {
            Ok(_size) => (),
            Err(_e) => return Err(TCPError::MalformedPacket)
        }

        match serde_json::from_slice::<Value>(&data_buffer) {
            Ok(value) => Frame::from_json(value),
            Err(_e) => Err(TCPError::MalformedJSON)
        }
    }

    pub async fn respond(&mut self, data: Value) -> Result<usize, TCPError> {
        let serialized = serde_json::to_string(&data).map_err(|_| TCPError::SerializeResponse)?;
        let mut serialized_as_bytes = serialized.as_bytes().to_vec();

        // There is no way this is the correct way to do this
        let mut bytes: Vec<u8> = vec![42];
        let res_length = serialized_as_bytes.len() as u16;
        let res_length_bytes: [u8; 2] = res_length.to_be_bytes();
        bytes.push(res_length_bytes[0]);
        bytes.push(res_length_bytes[1]);
        bytes.append(&mut serialized_as_bytes);

        self.stream.writable().await.map_err(|_| TCPError::ConnectionNotWritable)?;
        self.stream.try_write(&bytes).map_err(|_| TCPError::FailedWrite)
    }

    // pub async fn read_data(&mut self) -> Result<(), TCPError> {
    //     //let mut buf = Cursor::new(&self.buffer[..]);
    //     loop {
    //         // Wait for the socket to be readable
    //         match self.stream.readable().await {
    //             Ok(()) => (),
    //             Err(_) => return Err(UnknownError("Stream was not readable".to_string()))
    //         }
    //
    //         // Try to read data, this may still fail with `WouldBlock`
    //         // if the readiness event is a false positive.
    //         match self.stream.try_read(&mut self.buffer) {
    //             Ok(n) => {
    //                 self.buffer.truncate(n);
    //                 break;
    //             }
    //             Err(ref e) if e.kind() == WouldBlock => {
    //                 continue;
    //             }
    //             Err(_e) => {
    //                 return Err(UnknownError("Failed to read data from stream".to_string()));
    //             }
    //         }
    //     }
    //     println!("{:?}", self.buffer);
    // }
}
