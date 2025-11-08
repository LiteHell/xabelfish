mod screen_capture;

use std::io::*;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use clap::Parser;
use unix_socket_transport::unix_socket_client::UnixSocketClient;
use unix_socket_transport::unix_socket_server::UnixSocketServer;
use xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessage;
use xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType;

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
    let socket_path = Path::new(&args.socket_path);

    let mut socket = UnixSocketClient::connect(&socket_path).expect("Failed to bind socket path");
    println!("Connected screen capture control socket");

    // Loop for listening control command
    'listen_request_control: loop {
        // Handle invalid command
        let command = match socket.recv::<ContinuousCaptureMessage>() {
            Ok(parsed) => parsed,
            Err(err) => {
                println!(
                    "Control message parse error: {:#?}\nSending invalid message response!",
                    err
                );
                let _ = socket.send(&ContinuousCaptureMessage::invalid_message());
                continue 'listen_request_control;
            }
        };

        // Handle message
        let write_response_result: Result<()> = match command.message_type {
            ContinuousCaptureMessageType::StartCapture => {
                println!("Accepted start capture message on control");

                // Channel for initialization success retrival
                let (init_success_rw, init_success_recv) = mpsc::channel();

                // Create thread
                let _ = thread::spawn(move || {
                    let (mut data_server, data_server_sock_path) = match UnixSocketServer::create()
                    {
                        Ok(server) => server,
                        Err(_) => {
                            init_success_rw.send(None).unwrap();
                            return;
                        }
                    };

                    // Start capture
                    let mut capture = pipewire::PipeWireScreenCapture::new();
                    if let Err(_) = capture.start_capture() {
                        init_success_rw.send(None).unwrap();
                        return;
                    }

                    // Init success
                    init_success_rw
                        .send(Some(data_server_sock_path.clone()))
                        .unwrap();

                    'data_socket_listen_loop: loop {
                        match data_server.accept() {
                            Ok(mut data_client) => loop {
                                let image = capture.get_captured_image();
                                if let Some(image) = image {
                                    println!("Sending image...");
                                    let mut bytes: Vec<u8> = Vec::new();
                                    image
                                        .write_to(
                                            &mut Cursor::new(&mut bytes),
                                            image::ImageFormat::Png,
                                        )
                                        .expect("Failed to write png bytes");

                                    let image = ContinuousCaptureMessage::png(bytes);

                                    let _ = data_client.send(&image);
                                }
                            },
                            Err(_) => {
                                continue 'data_socket_listen_loop;
                            }
                        }
                    }
                });

                let init_success = init_success_recv.recv().unwrap();
                let response = if let Some(path) = init_success {
                    ContinuousCaptureMessage {
                        extra_bytes: vec![],
                        extra_str: path.clone(),
                        message_type: ContinuousCaptureMessageType::CaptureInitSuccess,
                    }
                } else {
                    ContinuousCaptureMessage {
                        extra_bytes: vec![],
                        extra_str: "".to_string(),
                        message_type: ContinuousCaptureMessageType::CaptureInitFail,
                    }
                };

                socket.send(&response)
            }
            ContinuousCaptureMessageType::Ping => {
                println!("Ping-pong on control socket!");
                socket.send(&ContinuousCaptureMessage::pong(command.extra_str))
            }
            _ => {
                println!("Ignoring Non-control message on control socket...");
                Ok(())
            }
        };

        // Listen for another command if write fails
        if write_response_result.is_err() {
            println!("Writing response failed... waiting for another request");
            continue 'listen_request_control;
        }
    }
}
