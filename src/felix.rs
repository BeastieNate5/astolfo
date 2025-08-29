use std::{env, io::{self, Read, Write}, net::TcpStream, process::{self, exit}};

use astolfo::CMD;

fn main() {
    let address = env::args().nth(1).unwrap();
    let config = bincode::config::standard();

    let mut stream = TcpStream::connect(address).unwrap_or_else(|err| {
        eprintln!("[\x1b[911mERR\x1b[0m] Failed to connect ({err})");
        process::exit(1);
    });

    loop {
        let mut size_buf = [0u8; 2];
        stream.read_exact(&mut size_buf).unwrap_or_else(|_| {
            eprintln!("[\x1b[911mERR\x1b[0m] Lost connection to astolfo");
            process::exit(1);
        });
        println!("Got size");

        let size = u16::from_be_bytes(size_buf);
        let mut cmd = vec![0u8; size as usize];
        println!("Size: {size}");

        stream.read_exact(cmd.as_mut_slice()).unwrap_or_else(|_| {
            eprintln!("[\x1b[911mERR\x1b[0m] Lost connection to astolfo");
            process::exit(1);
        });
        println!("Got cmd");

        let cmd = bincode::decode_from_slice::<CMD, _>(cmd.as_slice(), config);
        println!("decoded cmd");

        match cmd {
            Ok((cmd, _)) => match cmd {
                CMD::hello => {
                    if let Err(_) = send_hello(&mut stream) {
                        println!("[\x1b[91mERR\x1b[0m] Lost connection to astolfo");
                        process::exit(1);
                    }
                }
                _ => {}
            },
            Err(_) => {}
        }
    }
}

fn send_hello(stream: &mut TcpStream) -> io::Result<()> {
    let config = bincode::config::standard();
    let hello = bincode::encode_to_vec(CMD::hello, config).map_err(|err| {
        io::Error::new(io::ErrorKind::InvalidData, err)
    })?;
    
    let size = &(hello.len() as u16).to_be_bytes();
    stream.write_all(size)?;
    stream.write_all(hello.as_slice())?;
    Ok(())
}
