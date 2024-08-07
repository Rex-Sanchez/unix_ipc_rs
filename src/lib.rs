use std::{
    fs::remove_file,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
};

use serde::{de::DeserializeOwned, Serialize};

mod error;

pub use crate::error::Result;

pub struct SocketServer {
    socket: UnixStream,
    addr: &'static str,
}

impl SocketServer {
    pub fn new(addr: &'static str) -> Result<Self> {
        let listener = UnixListener::bind(addr)?;
        let (socket, _) = listener.accept()?;

        Ok(Self { socket, addr })
    }
    pub fn recv<T: DeserializeOwned>(&mut self) -> Option<T> {
        // create fixed 4 byte buffer;
        let mut buf = [0u8; 4];

        // read 4 bytes of data;
        self.socket.read_exact(&mut buf).ok()?;

        // get len of message;
        let message_len = u32::from_be_bytes(buf);

        // create buffer with message len;
        let mut message_buf = vec![0u8; message_len as usize];

        // read from socket until buffer is full
        self.socket.read_exact(&mut message_buf).ok()?;

        // deserialize the message
        let message = bincode::deserialize::<T>(&message_buf).ok()?;

        return Some(message);
    }
    pub fn recv_blocking<T: DeserializeOwned>(&mut self) -> T {
        loop {
            if let Some(message) = self.recv() {
                return message;
            }
        }
    }
    pub fn send<T: Serialize>(&mut self, message: T) -> Option<()> {
        // encode message;
        let message = bincode::serialize(&message).ok()?;
        // get len of message;
        let message_len = message.len();

        // send len of message;
        self.socket.write_all(&message_len.to_be_bytes()).ok()?;
        // send message;
        self.socket.write_all(&message).ok()?;

        Some(())
    }
}

impl Drop for SocketServer {
    fn drop(&mut self) {
        self.socket.shutdown(std::net::Shutdown::Both).ok();
        remove_file(&self.addr).ok();
    }
}

pub struct SocketClient {
    socket: UnixStream,
}

impl SocketClient {
    pub fn new(addr: &str) -> crate::error::Result<Self> {
        let socket = UnixStream::connect(addr)?;

        Ok(Self { socket })
    }

    pub fn recv<T: DeserializeOwned>(&mut self) -> Option<T> {
        // create a fixed size 4 byte buffer;
        let mut buf = [0u8; 4];

        // receiver message size;
        self.socket.read_exact(&mut buf).ok()?;

        // convert buffer to u32;
        let message_len = u32::from_be_bytes(buf);

        // create buffer of size message size;
        let mut message_buf = vec![0u8; message_len as usize];

        // read from socket until buffer is full;
        self.socket.read_exact(&mut message_buf).ok()?;

        // deserialize the message into struct;
        let message = bincode::deserialize::<T>(&message_buf).ok()?;

        return Some(message);
    }

    pub fn recv_blocking<T: DeserializeOwned>(&mut self) -> T {
        loop {
            if let Some(message) = self.recv() {
                return message;
            }
        }
    }

    pub fn send<T: Serialize>(&mut self, message: T) -> Option<()> {
        // encode message;
        let message = bincode::serialize(&message).ok()?;
        // get len of message;
        let message_len = (message.len() as u32).to_be_bytes();

        // send len of message;
        self.socket.write_all(&message_len).ok();
        // send message;
        self.socket.write_all(&message).ok();

        Some(())
    }
}
