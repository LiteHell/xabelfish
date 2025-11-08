use std::{fs::remove_file, io::Error, os::unix::net::UnixListener, path::Path};

use tempfile::NamedTempFile;

use crate::unix_socket_client::UnixSocketClient;

pub struct UnixSocketServer {
    listener: UnixListener,
}

impl UnixSocketServer {
    pub fn bind(path: &Path) -> Result<Self, Error> {
        let stream = UnixListener::bind(&path)?;
        Ok(Self { listener: stream })
    }

    pub fn create() -> Result<(Self, String), Error> {
        let temp_path = NamedTempFile::with_suffix(".sock")
            .expect("Failed to create temp file")
            .into_temp_path();

        remove_file(&temp_path).expect("Failed to delete temp file");

        let server = Self::bind(&temp_path)?;
        let path_string = temp_path.as_os_str().to_str().unwrap().to_string();

        Ok((server, path_string))
    }

    pub fn accept(&mut self) -> Result<UnixSocketClient, Error> {
        let (stream, _addr) = self.listener.accept()?;

        Ok(UnixSocketClient::from(stream))
    }
}
