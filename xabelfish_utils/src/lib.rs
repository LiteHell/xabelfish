use std::{io::Read, os::unix::net::UnixStream};

pub fn read_until_nul_from_unix_stream(stream: &mut UnixStream, buffer: &mut String) -> Result<(), std::io::Error> {
    let mut vec = vec![];
    let mut buf = [0; 256];
    loop {
        let read = stream.read(&mut buf)?;
        
        let mut terminated = false;
        for i in 0..read {
            if buf[i] == 0 {
                terminated = true;
            }
        }

        vec.extend_from_slice(&buf[0..read]);
        if terminated {
            vec.pop(); // pop nul terminator
            *buffer = String::from_utf8(vec).expect("Failed to parse utf8 string");
            break;
        }
    }

    Ok(())
}
