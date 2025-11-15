use std::{path::PathBuf, process::Command};

use rustix::process::{Pid, Signal, kill_process};
use xabelfish_unix_socket::unix_socket_server::UnixSocketServer;

pub(super) struct ListenerPidAndSockPath<T>
where
    T: Clone,
{
    process: Pid,
    socket_path: String,
    extra: T,
}

impl<T> ListenerPidAndSockPath<T>
where
    T: Clone,
{
    pub fn none() -> Option<Self> {
        None
    }

    pub fn create_with_process(exec_path: PathBuf, extra: T) -> (UnixSocketServer, Self) {
        let (translate_listener, socket_path) =
            UnixSocketServer::create().expect("Failed to create control socket for translate");

        let process = Command::new(exec_path.as_os_str())
            .arg("--socket-path")
            .arg(socket_path.clone())
            .spawn()
            .expect("Failed to run translate");

        (
            translate_listener,
            Self {
                process: (Pid::from_child(&process)),
                extra: (extra),
                socket_path: (socket_path),
            },
        )
    }

    pub fn extra(&self) -> T {
        self.extra.clone()
    }

    pub fn change_process(&mut self, exec_path: PathBuf, new_extra: T) {
        self.kill(false);

        let socket_path = self.socket_path.clone();

        let process = Command::new(exec_path.as_os_str())
            .arg("--socket-path")
            .arg(socket_path.clone())
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
