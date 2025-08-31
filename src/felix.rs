use std::{env, io::{Read, Write}, net::TcpStream, process, thread::sleep, time::Duration};

use astolfo::FemState;

fn main() {
    let address = env::args().nth(1).unwrap();
    let config = bincode::config::standard();

    let mut stream = TcpStream::connect(address).unwrap_or_else(|err| {
        eprintln!("[\x1b[911mERR\x1b[0m] Failed to connect ({err})");
        process::exit(1);
    });

    loop {
        let hello_msg = b"meow";
        let hello_size : [u8; 2] = (hello_msg.len() as u16).to_be_bytes();

        stream.write_all(&hello_size).unwrap_or_else(|_| {
            eprintln!("[\x1b[911mERR\x1b[0m] Lost connection to astolfo");
            process::exit(1);
        });

        stream.write_all(hello_msg).unwrap_or_else(|_| {
            eprintln!("[\x1b[911mERR\x1b[0m] Lost connection to astolfo");
            process::exit(1);
        });

        let mut size_buf = [0u8; 2];
        stream.read_exact(&mut size_buf).unwrap_or_else(|_| {
            eprintln!("[\x1b[911mERR\x1b[0m] Lost connection to astolfo");
            process::exit(1);
        });
        println!("Got size");

        let size = u16::from_be_bytes(size_buf);
        let mut state = vec![0u8; size as usize];
        println!("Size: {size}");

        stream.read_exact(state.as_mut_slice()).unwrap_or_else(|_| {
            eprintln!("[\x1b[911mERR\x1b[0m] Lost connection to astolfo");
            process::exit(1);
        });
        println!("Got cmd");

        let cmd = bincode::decode_from_slice::<FemState, _>(state.as_slice(), config);
        println!("decoded cmd");

        match cmd {
            Ok((cmd, _)) => match cmd {
                FemState::Idle => {
                    println!("CMD: Idle");
                }
                _ => {}
            },
            Err(_) => {}
        }

        sleep(Duration::from_secs(3));

    }
}

