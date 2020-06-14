extern crate bincode;

use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

pub trait MessageStream {
    fn write_msg<T: ?Sized>(&mut self, value: &T) -> Result<usize, io::Error>
    where
        T: serde::Serialize;
    fn read_msg<'a, T: ?Sized + Clone>(&mut self, buf: &'a mut Vec<u8>) -> Result<T, io::Error>
    where
        T: serde::Deserialize<'a>;
}

impl MessageStream for TcpStream {
    /// Write message to the stream.
    fn write_msg<T: ?Sized>(&mut self, value: &T) -> Result<usize, io::Error>
    where
        T: serde::Serialize,
    {
        match bincode::serialize(&value) {
            Ok(mut serialized_value) => {
                let mut data = (serialized_value.len() as u16)
                    .to_be_bytes()
                    .to_vec()
                    .to_owned();
                data.append(&mut serialized_value);
                match self.write(&data) {
                    Ok(size) => Ok(size),
                    Err(e) => Err(io::Error::new(
                        e.kind(),
                        format!("Failed to write frame: {:?}", e),
                    )),
                }
            }
            Err(e) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to serialize frame: {:?}", e),
            )),
        }
    }

    /// Read message from the stream.
    ///
    /// TODO: fight the borrow checker and initialize the buffer within a function.
    fn read_msg<'a, T: ?Sized + Clone>(&mut self, buf: &'a mut Vec<u8>) -> Result<T, io::Error>
    where
        T: serde::Deserialize<'a>,
    {
        let mut body_len_buf = [0 as u8; 2];
        match self.read_exact(&mut body_len_buf) {
            Ok(_) => {
                let body_len = ((body_len_buf[0] as u16) << 8) | body_len_buf[1] as u16;

                buf.clear();
                buf.extend(vec![0; body_len as usize]);

                match self.read_exact(buf) {
                    Ok(_) => match bincode::deserialize::<T>(buf) {
                        Ok(message) => Ok(message),
                        Err(e) => Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!("Failed to deserialize frame body: {:?}", e),
                        )),
                    },
                    Err(e) => Err(io::Error::new(
                        e.kind(),
                        format!("Failed to read frame body: {:?}", e),
                    )),
                }
            }
            Err(e) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read frame header: {:?}", e),
            )),
        }
    }
}
