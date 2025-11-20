use std::{
    fs::{File, remove_file},
    path::PathBuf,
    process::Command,
};

use rustix::process::{Pid, Signal, kill_process};
use tempfile::{NamedTempFile, tempfile};
use xabelfish_unix_socket::unix_socket_server::UnixSocketServer;

pub(super) struct ListenerPidAndSockPath<T>
where
    T: Clone,
{
    process: Pid,
    socket_path: String,
    lockfile: PathBuf,
    extra: T,
}

impl<T> ListenerPidAndSockPath<T>
where
    T: Clone,
{
    pub fn none() -> Option<Self> {
        None
    }

    fn create_temp_lockfile() -> PathBuf {
        let mut temp_lockfile = NamedTempFile::new().unwrap();
        temp_lockfile.disable_cleanup(true);

        temp_lockfile.into_temp_path().to_path_buf()
    }

    pub fn create_with_process(exec_path: PathBuf, extra: T) -> (UnixSocketServer, Self) {
        let (translate_listener, socket_path) =
            UnixSocketServer::create().expect("Failed to create control socket for translate");
        let lockfile = ListenerPidAndSockPath::<T>::create_temp_lockfile();

        let process = Command::new(exec_path.as_os_str())
            .arg("--socket-path")
            .arg(socket_path.clone())
            .arg("--lock-file")
            .arg(lockfile.clone())
            .spawn()
            .expect("Failed to run translate");

        (
            translate_listener,
            Self {
                process: (Pid::from_child(&process)),
                extra: (extra),
                socket_path: (socket_path),
                lockfile: lockfile,
            },
        )
    }

    pub fn is_alive(&self) -> bool {
        let lockfile = File::open(self.lockfile.clone());

        if let Ok(lockfile) = lockfile {
            lockfile.try_lock().is_err()
        } else {
            false
        }
    }

    pub fn extra(&self) -> T {
        self.extra.clone()
    }

    pub fn change_process(&mut self, exec_path: PathBuf, new_extra: T) {
        self.kill(false);

        let socket_path = self.socket_path.clone();
        let lockfile = self.lockfile.clone();

        let process = Command::new(exec_path.as_os_str())
            .arg("--socket-path")
            .arg(socket_path.clone())
            .arg("--lock-file")
            .arg(lockfile.clone())
            .spawn()
            .expect("Failed to run translate");

        self.process = Pid::from_child(&process);
        self.extra = new_extra;
    }

    pub fn kill(&mut self, sigkill: bool) -> rustix::io::Result<()> {
        kill_process(
            self.process,
            if sigkill { Signal::KILL } else { Signal::TERM },
        )?;

        Ok(())
    }
}

impl<T> Drop for ListenerPidAndSockPath<T>
where
    T: Clone,
{
    fn drop(&mut self) {
        remove_file(self.lockfile.clone());
    }
}
