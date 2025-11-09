use std::{
    fs::{exists, remove_file},
    io::Error,
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

use crate::unix_socket_client::UnixSocketClient;

pub struct UnixSocketServer {
    listener: UnixListener,
    path: PathBuf,
}

impl UnixSocketServer {
    pub fn bind(path: &Path) -> Result<Self, Error> {
        let listener = UnixListener::bind(path)?;
        println!("Server socket bound at {listener:?}");

        Ok(Self {
            listener,
            path: path.to_path_buf(),
        })
    }

    pub fn get_temp_sock_path() -> PathBuf {
        let temp_path = NamedTempFile::with_suffix(".sock")
            .expect("Failed to create temp file")
            .into_temp_path();

        remove_file(&temp_path).expect("Failed to delete temp file");

        // remove_file doesn't gauarantee that the file will be deleted immediately...
        // so we need to wait for file to be deleted.
        while exists(&temp_path).is_ok_and(|x| x) {}

        temp_path.to_path_buf()
    }

    pub fn create() -> Result<(Self, String), Error> {
        let temp_path = Self::get_temp_sock_path();

        let server = Self::bind(&temp_path)?;
        let path_string = temp_path.as_os_str().to_str().unwrap().to_string();

        Ok((server, path_string))
    }

    pub fn accept(&mut self) -> Result<UnixSocketClient, Error> {
        println!("Accepting... in accept func");
        let (stream, _addr) = self.listener.accept()?;
        println!("Accepted... in accept func");

        Ok(UnixSocketClient::from(stream))
    }
}

impl Drop for UnixSocketServer {
    fn drop(&mut self) {
        let _ = remove_file(self.path.as_path());
        println!("server dropped");
    }
}
