use std::{fs::remove_file, os::unix::net::UnixListener, path::Path, process::Command};

use unix_socket_transport::{
    unix_socket_client::UnixSocketClient, unix_socket_server::UnixSocketServer,
};
use xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessage;

fn main() {
    remove_file("/tmp/sibal.sock");
    let sibal = UnixListener::bind("/tmp/sibal.sock").unwrap();
    println!("sibal: {sibal:#?}");

    println!("Note: cli is for testing");

    let (mut cont_capture_listener, control_sock_path) =
        UnixSocketServer::create().expect("Failed to create control socket");

    println!("cont capture listening sock: {:#?}", &control_sock_path);

    let cont_capture_process = Command::new("./xabelfish_cont_capture")
        .arg("--socket-path")
        .arg(control_sock_path)
        .spawn()
        .expect("Failed to run continous capture");

    loop {
        println!("Accepting...");
        let mut cont_capture = cont_capture_listener
            .accept()
            .expect("Failed to accept continous capture connection");

        println!("Accepted cont capture connection");
        println!("Sending capture init command...");
        cont_capture.send(&ContinuousCaptureMessage {
            message_type:
                xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::StartCapture,
            extra_bytes: vec![],
            extra_str: String::new(),
        }).expect("Failed to send init command");

        let response: ContinuousCaptureMessage =
            cont_capture.recv().expect("Failed to receive response");

        match response.message_type {
            xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::CaptureInitFail => {
                println!("Oops, failed to init capture!");
            }
            xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::CaptureInitSuccess => {
                println!("Connecting data socket");

                let mut data_client = UnixSocketClient::connect(&Path::new(&response.extra_str)).expect("Failed to connect data socket");
                loop {
                    let data: ContinuousCaptureMessage = data_client.recv().expect("Failed to get data");

                    println!("Got message: {:#?} type", data.message_type);
                }
            }
            _ => {}
        }
    }
}
