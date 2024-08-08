use std::{
    fs::remove_file,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
};

use serde::{de::DeserializeOwned, Serialize};

mod error;

pub use crate::error::Result;

pub struct IPCSocket {
    socket: UnixStream,
    addr: &'static str,
}

impl IPCSocket {
    pub fn new_server(addr: &'static str) -> Result<Self> {
        let listener = UnixListener::bind(addr)?;
        let (socket, _) = listener.accept()?;
        Ok(Self { socket, addr })
    }

    pub fn new_client(addr: &'static str) -> Result<Self> {
        let socket = UnixStream::connect(addr)?;
        Ok(Self { socket, addr })
    }

    fn receive_data(&mut self, buffer: &mut [u8]) -> Result<Option<()>> {
        if let Err(e) = self.socket.read_exact(buffer) {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                // if the buffer is filled with 0x00 return a Ok(None);
                return Ok(None);
            } else {
                return Err(e.into());
            }
        };
        Ok(Some(()))
    }
    pub fn recv<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        let mut buf = [0u8; 4];

        if self.receive_data(&mut buf)?.is_none() {
            return Ok(None);
        }

        let mut message_buf = vec![0u8; u32::from_be_bytes(buf) as usize];

        if self.receive_data(&mut message_buf)?.is_none() {
            return Ok(None);
        }

        let message = bincode::deserialize::<T>(&message_buf)?;

        return Ok(Some(message));
    }

    pub fn recv_blocking<T: DeserializeOwned>(&mut self) -> Result<T> {
        loop {
            if let Some(message) = self.recv()? {
                return Ok(message);
            }
        }
    }

    pub fn send<T: Serialize>(&mut self, message: T) -> Result<()> {
        // encode message;
        let message = bincode::serialize(&message)?;
        // get len of message;
        let message_len = message.len();

        // send len of message;
        self.socket.write_all(&message_len.to_be_bytes())?;
        // send message;
        self.socket.write_all(&message)?;

        Ok(())
    }
}

impl Drop for IPCSocket {
    fn drop(&mut self) {
        self.socket.shutdown(std::net::Shutdown::Both).ok();
        remove_file(self.addr).ok();
    }
}



#[cfg(test)]
mod test {
    use std::thread::spawn;
    use crate::IPCSocket;

    #[test]
    fn t1() {
        let a = spawn(|| {
            let mut s = IPCSocket::new_server("/tmp/sock.sock").unwrap();
            let m: u32 = s.recv_blocking().unwrap();
            assert_eq!(m, 69);
            s.send::<u32>(42).ok();
        });

        let b = spawn(|| {
            let mut c = IPCSocket::new_client("/tmp/sock.sock").unwrap();
            c.send::<u32>(69).ok();
            let m = c.recv_blocking::<u32>().unwrap();
            assert_eq!(m, 42);
        });

        a.join().ok();
        b.join().ok();
    }
}
