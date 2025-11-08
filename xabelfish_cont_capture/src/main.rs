mod screen_capture;

use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;
use std::{io::*, os::unix::net::UnixListener};

use clap::Parser;
use screen_continuous_capture_protocol::Message;
use screen_continuous_capture_protocol::MessageType;

use crate::screen_capture::Capture;
use crate::screen_capture::pipewire::pipewire;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandArgs {
    #[arg(short, long)]
    socket_path: String,

    #[arg(short, long, default_value_t = false)]
    exit_on_control_disconnect: bool,
}

fn main() {
    let args = CommandArgs::parse();

    let listener = UnixListener::bind(args.socket_path).expect("Failed to bind socket path");

    // Loop for accepting control socket
    'accept_control: loop {
        match listener.accept() {
            Ok((mut socket, addr)) => {
                println!("Got a control client: {:?} - {:?}", socket, addr);
                let mut command = String::new();

                // Loop for listening control command
                'listen_control: loop {
                    let read_request_result = socket.read_to_string(&mut command);

                    match read_request_result {
                        // Handle control client disconnection
                        Err(err) => {
                            if args.exit_on_control_disconnect {
                                println!(
                                    "Dropping control socket due to read error on listener socket: {:#?}",
                                    &err
                                );
                                return;
                            } else {
                                println!(
                                    "Looking for new control socket due to read error on listener socket: {:#?}",
                                    &err
                                );
                                continue 'accept_control;
                            }
                        }

                        // Handle message retrival
                        Ok(_) => {
                            // Handle invalid command
                            let command =
                                match screen_continuous_capture_protocol::Message::parse_from_str(
                                    command.as_str(),
                                ) {
                                    Ok(parsed) => parsed,
                                    Err(err) => {
                                        println!(
                                            "Control message parse error: {:#?}\nSending invalid message response!",
                                            err
                                        );
                                        let _ = socket.write_all(
                                            Message::invalid_message().to_string().as_bytes(),
                                        );
                                        continue 'listen_control;
                                    }
                                };

                            // Handle message
                            let write_response_result: Result<()> = match command.message_type {
                                MessageType::StartCapture => {
                                    println!("Accepted start capture message on control");

                                    // Create temp socket file path
                                    let data_socket = tempfile::NamedTempFile::with_suffix(".sock")
                                        .expect("Failed to create temp file for data socket");
                                    let data_socket_path = data_socket.into_temp_path();
                                    let data_socket_path_str =
                                        data_socket_path.as_os_str().to_str().unwrap().to_string();

                                    // Channel for initialization success retrival
                                    let (init_success_rw, init_success_recv) = mpsc::channel();

                                    // Create thread
                                    let _ = thread::spawn(move || {
                                        // Create data socket
                                        let listener = if let Ok(socket) =
                                            UnixListener::bind(&data_socket_path)
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

                                                        if Message::png(bytes)
                                                            .write_to(&mut socket)
                                                            .is_err()
                                                        {
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
                                        Message {
                                            extra_bytes: vec![],
                                            extra_str: data_socket_path_str.clone(),
                                            message_type: MessageType::CaptureInitSuccess,
                                        }
                                    } else {
                                        Message {
                                            extra_bytes: vec![],
                                            extra_str: "".to_string(),
                                            message_type: MessageType::CaptureInitFail,
                                        }
                                    };
                                    socket.write_all(response.to_string().as_bytes())
                                }
                                MessageType::Ping => {
                                    println!("Ping-pong on control socket!");
                                    socket.write_all(
                                        Message::pong(command.extra_str).to_string().as_bytes(),
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
                    }
                }
            }
            Err(e) => println!("control accept function failed: {:?}", e),
        }
    }
}
