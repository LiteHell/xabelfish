use std::{
    cmp,
    io::{Error, Read, Write},
    os::unix::net::UnixStream,
    path::Path,
};

use serde::{Serialize, de::DeserializeOwned};

pub struct UnixSocketClient {
    stream: UnixStream,
}

const READ_BUFFER_SIZE: usize = 1024;

impl From<UnixStream> for UnixSocketClient {
    fn from(stream: UnixStream) -> Self {
        Self { stream }
    }
}

impl UnixSocketClient {
    pub fn connect(path: &Path) -> Result<Self, Error> {
        let stream = UnixStream::connect(&path)?;
        Ok(Self { stream })
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

    pub fn recv<'de, T>(&mut self) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let data = self.recv_vec()?;
        let slice = data.as_slice();

        let deserialized = bson::deserialize_from_slice(slice).expect("Failed to deserialize bson");
        Ok(deserialized)
    }

    fn recv_data_len(&mut self) -> Result<usize, Error> {
        let len_byte_count: usize = (usize::BITS / 8).try_into().unwrap();
        let mut buffer = [0; 8];
        let mut total_read_byte_count: usize = 0;

        while total_read_byte_count < len_byte_count {
            let byte_count_read = self
                .stream
                .read(&mut buffer[total_read_byte_count..len_byte_count])?;
            total_read_byte_count += byte_count_read;
        }

        Ok(usize::from_ne_bytes(buffer))
    }

    fn recv_to(&mut self, vec: &mut Vec<u8>) -> Result<(), Error> {
        let data_len = self.recv_data_len()?;
        let mut total_read_byte_count: usize = 0;

        while total_read_byte_count < data_len {
            let remaining_byte_count = data_len - total_read_byte_count;
            let mut buffer = [0; READ_BUFFER_SIZE];
            let bytes_to_read = cmp::min(READ_BUFFER_SIZE, remaining_byte_count);

            let read_byte_count = self.stream.read(&mut buffer[0..bytes_to_read])?;

            total_read_byte_count += read_byte_count;
            vec.extend_from_slice(&buffer[0..read_byte_count]);
        }

        Ok(())
    }

    fn recv_vec(&mut self) -> Result<Vec<u8>, Error> {
        let mut vec = Vec::new();
        self.recv_to(&mut vec)?;

        Ok(vec)
    }
}
