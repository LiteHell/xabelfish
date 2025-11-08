mod screen_capture;

use std::fs::remove_file;
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;
use std::{io::*, os::unix::net::UnixListener};

use clap::Parser;
use image::EncodableLayout;
use xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType;
use xabelfish_socket_protocol::cont_capture::{ContinuousCaptureMessage};
use xabelfish_utils::read_until_nul_from_unix_stream;

use crate::screen_capture::Capture;
use crate::screen_capture::pipewire::pipewire;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandArgs {
    #[arg(short, long)]
    socket_path: String,
}

fn main() {
    let args = CommandArgs::parse();

    let mut socket = UnixStream::connect(args.socket_path).expect("Failed to bind socket path");
    println!("Connected screen capture control socket");

    let mut command = String::new();

    // Loop for listening control command
    'listen_control: loop {
        let read_request_result = read_until_nul_from_unix_stream(&mut socket, &mut command);
        println!("Read capture request message... raw: {:#?}", &command);

        match read_request_result {
            // Handle control socket disconnection
            Err(err) => {
                println!(
                    "Exiting control socket due to read error on listener socket: {:#?}",
                    &err
                );
                return;
            }

            // Handle message retrival
            Ok(_) => {
                // Handle invalid command
                let command = match xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessage::parse_from_str(
                    command.as_str(),
                ) {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        println!(
                            "Control message parse error: {:#?}\nSending invalid message response!",
                            err
                        );
                        let _ = socket
                            .write_all(ContinuousCaptureMessage::invalid_message().to_nul_ended_vec().as_bytes());
                        continue 'listen_control;
                    }
                };

                // Handle message
                let write_response_result: Result<()> = match command.message_type {
                    ContinuousCaptureMessageType::StartCapture => {
                        println!("Accepted start capture message on control");

                        // Create temp socket file path
                        let data_socket = tempfile::NamedTempFile::with_suffix(".sock")
                            .expect("Failed to create temp file for data socket");
                        let data_socket_path = data_socket.into_temp_path();
                        remove_file(&data_socket_path)
                            .expect("FAiled to delete file for data socket");
                        let data_socket_path_str =
                            data_socket_path.as_os_str().to_str().unwrap().to_string();

                        // Channel for initialization success retrival
                        let (init_success_rw, init_success_recv) = mpsc::channel();

                        // Create thread
                        let _ = thread::spawn(move || {
                            // Create data socket
                            let listener = if let Ok(socket) = UnixListener::bind(&data_socket_path)
                            {
                                socket
                            } else {
                                init_success_rw.send(false).unwrap();
                                return;
                            };

                            // Start capture
                            let mut capture = pipewire::PipeWireScreenCapture::new();
                            if let Err(_) = capture.start_capture() {
                                init_success_rw.send(false).unwrap();
                                return;
                            }

                            // Init success
                            init_success_rw.send(true).unwrap();

                            'data_socket_listen_loop: loop {
                                match listener.accept() {
                                    Ok((mut socket, addr)) => loop {
                                        let image = capture.get_captured_image();
                                        if let Some(image) = image {
                                            println!(
                                                "Sending image to {:#?} at {:#?}",
                                                addr,
                                                SystemTime::now()
                                            );
                                            let mut bytes: Vec<u8> = Vec::new();
                                            image
                                                .write_to(
                                                    &mut Cursor::new(&mut bytes),
                                                    image::ImageFormat::Png,
                                                )
                                                .expect("Failed to write png bytes");

                                            let mut disconnected = false;
                                            if ContinuousCaptureMessage::png(bytes).write_to(&mut socket).is_err() {
                                                disconnected = true;
                                            }

                                            if socket.write(&[0]).is_err() {
                                                disconnected = true;
                                            }
                                            if disconnected {
                                                println!(
                                                    "Looks like data socket is disconnected. DROP"
                                                );
                                                // Drop data socket
                                                return;
                                            }
                                        }
                                    },
                                    Err(_) => {
                                        continue 'data_socket_listen_loop;
                                    }
                                }
                            }
                        });

                        let init_success = init_success_recv.recv().unwrap();
                        let response = if init_success {
                            ContinuousCaptureMessage {
                                extra_bytes: vec![],
                                extra_str: data_socket_path_str.clone(),
                                message_type: ContinuousCaptureMessageType::CaptureInitSuccess,
                            }
                        } else {
                            ContinuousCaptureMessage {
                                extra_bytes: vec![],
                                extra_str: "".to_string(),
                                message_type: ContinuousCaptureMessageType::CaptureInitFail,
                            }
                        };
                        socket.write_all(response.to_nul_ended_vec().as_bytes())
                    }
                    ContinuousCaptureMessageType::Ping => {
                        println!("Ping-pong on control socket!");
                        socket.write_all(
                            ContinuousCaptureMessage::pong(command.extra_str)
                                .to_nul_ended_vec()
                                .as_bytes(),
                        )
                    }
                    _ => {
                        println!("Ignoring Non-control message on control socket...");
                        Ok(())
                    }
                };

                // Listen for another command if write fails
                if write_response_result.is_err() {
                    println!("Writing response failed... waiting for another request");
                    continue 'listen_control;
                }
            }
            Err(e) => println!("control accept function failed: {:?}", e),
        }
    }
}
