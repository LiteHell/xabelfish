use std::{
    cmp,
    io::{Error, Read, Write},
    os::unix::net::UnixStream,
    path::Path,
};

use serde::{Serialize, de::DeserializeOwned};

pub struct UnixSocketClient {
    stream: UnixStream,
    closed: bool,
}

const READ_BUFFER_SIZE: usize = 1024;

impl From<UnixStream> for UnixSocketClient {
    fn from(stream: UnixStream) -> Self {
        Self {
            stream,
            closed: false,
        }
    }
}

impl UnixSocketClient {
    pub fn connect(path: &Path) -> Result<Self, Error> {
        let stream = UnixStream::connect(&path)?;
        Ok(Self {
            stream,
            closed: false,
        })
    }

    fn send_bytes(&mut self, data: &[u8]) -> Result<usize, Error> {
        let data_len = data.len();
        self.stream.write_all(&data_len.to_ne_bytes())?;
        self.stream.write_all(data)?;

        Ok(data_len)
    }

    pub fn send<T: Serialize>(&mut self, data: &T) -> Result<(), Error> {
        let serialized = bson::serialize_to_vec(data).expect("Failed to serailize to bson");

        self.send_bytes(serialized.as_slice())?;
        Ok(())
    }

    pub fn recv<'de, T>(&mut self) -> Result<Option<T>, Error>
    where
        T: DeserializeOwned,
    {
        let data = {
            let recv_result = self.recv_vec()?;
            if let Some(vec) = recv_result {
                vec
            } else {
                return Ok(None);
            }
        };

        let slice = data.as_slice();

        let deserialized = bson::deserialize_from_slice(slice).expect("Failed to deserialize bson");
        Ok(Some(deserialized))
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    fn recv_data_len(&mut self) -> Result<Option<usize>, Error> {
        let len_byte_count: usize = (usize::BITS / 8).try_into().unwrap();
        let mut buffer = [0; 8];
        let mut total_read_byte_count: usize = 0;

        while total_read_byte_count < len_byte_count {
            let byte_count_read = self
                .stream
                .read(&mut buffer[total_read_byte_count..len_byte_count])?;

            if byte_count_read == 0 {
                self.closed = true;
                return Ok(None);
            }

            total_read_byte_count += byte_count_read;
        }

        Ok(Some(usize::from_ne_bytes(buffer)))
    }

    fn recv_to(&mut self, vec: &mut Vec<u8>) -> Result<Option<()>, Error> {
        let data_len_option = self.recv_data_len()?;
        let data_len = if let Some(data_len) = data_len_option {
            data_len
        } else {
            return Ok(None);
        };
        let mut total_read_byte_count: usize = 0;

        while total_read_byte_count < data_len {
            let remaining_byte_count = data_len - total_read_byte_count;
            let mut buffer = [0; READ_BUFFER_SIZE];
            let bytes_to_read = cmp::min(READ_BUFFER_SIZE, remaining_byte_count);

            let read_byte_count = self.stream.read(&mut buffer[0..bytes_to_read])?;

            if read_byte_count == 0 {
                self.closed = true;
                return Ok(None);
            }

            total_read_byte_count += read_byte_count;
            vec.extend_from_slice(&buffer[0..read_byte_count]);
        }

        Ok(Some(()))
    }

    fn recv_vec(&mut self) -> Result<Option<Vec<u8>>, Error> {
        let mut vec = Vec::new();
        let success = self.recv_to(&mut vec)?;

        if success.is_some() {
            Ok(Some(vec))
        } else {
            Ok(None)
        }
    }
}

impl Drop for UnixSocketClient {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}
