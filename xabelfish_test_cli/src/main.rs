use std::{
    fs, io::{Read, Write}, os::unix::net::{UnixListener, UnixStream}, process
};

use xabelfish_socket_protocol::{MessageType, read_line_for_unix_stream};

fn main() {
    println!("Note: cli is for testing only");
    let sock_file = tempfile::NamedTempFile::with_suffix(".sock").unwrap();
    let sock_path = sock_file.into_temp_path();
    fs::remove_file(&sock_path).unwrap();
    println!("Socket path: {:#?}", sock_path);

    let mut listener = UnixListener::bind(&sock_path).unwrap();

    println!("Control sock listening start...");

    let contcap = process::Command::new("./xabelfish_cont_capture")
        .arg("--socket-path")
        .arg(sock_path.as_os_str().to_str().unwrap().to_string())
        .spawn()
        .expect("Failed to start continousous capture");
    
    let (mut stream, _) = listener.accept().unwrap();

    println!("accepted cont-cap sock connect");

    loop {
        stream.write_all(
            (xabelfish_socket_protocol::Message {
                message_type: xabelfish_socket_protocol::MessageType::StartCapture,
                extra_bytes: vec![],
                extra_str: String::new(),
            })
            .to_string()
            .as_bytes(),
        ).unwrap();
        stream.write(&[0]).unwrap();

        let mut response_str = String::new();
        read_line_for_unix_stream(&mut stream, &mut response_str).unwrap();

        let response = xabelfish_socket_protocol::Message::from(response_str);
        println!("Control response: {:#?}", response);

        if matches!(response.message_type, MessageType::CaptureInitSuccess) {
            let data_socket_path = response.extra_str;
            println!("Data socket path: {:#?}", data_socket_path);
            let mut stream = UnixStream::connect(data_socket_path).unwrap();

            loop {
                let mut response_str = String::new();
                read_line_for_unix_stream(&mut stream, &mut response_str).unwrap();

               // let response = xabelfish_socket_protocol::Message::from(response_str);
                //println!("Data response: {:#?}", response);
            }
        }
    }
}
