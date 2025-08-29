use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    process,
    sync::{Arc, Mutex},
    time::Duration,
};

use astolfo::CMD;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    time::sleep,
};

#[derive(Debug)]
enum FemState {
    Idle,
    Attacking,
    Dead,
}

#[derive(Debug)]
struct Femboy {
    addr: SocketAddr,
    status: FemState,
}

#[tokio::main]
async fn main() {
    let port = env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: astolfo <PORT>");
        process::exit(1);
    });
    let port: u16 = port.parse().unwrap_or_else(|_err| {
        println!("[\x1b[91mERR\x1b[0m] Please provide valid port number");
        process::exit(1);
    });

    let femtable: Arc<Mutex<HashMap<u16, Femboy>>> = Arc::new(Mutex::new(HashMap::new()));
    let fem_counter: Arc<Mutex<u16>> = Arc::new(Mutex::new(0));

    let listen_femtable = Arc::clone(&femtable);
    let listen_fem_counter = Arc::clone(&fem_counter);

    let listen_task = tokio::spawn(async move {
        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .unwrap_or_else(|err| {
                println!("[\x1b[91mERR\x1b[0m] Failed to bind address ({err})");
                process::exit(1);
            });

        println!("[\x1b[94mINFO\x1b[0m] Listening for femboys on 0.0.0.0:{port}");

        loop {
            let femboy = match listener.accept().await {
                Ok((stream, addr)) => (stream, addr),
                Err(err) => {
                    println!("[\x1b[91mERR\x1b[0m] Failed to transition a femboy ({err})");
                    continue;
                }
            };

            {
                let mut femtable = listen_femtable.lock().unwrap_or_else(|_| {
                    process::exit(1);
                });

                let mut fem_counter = listen_fem_counter.lock().unwrap_or_else(|_| {
                    process::exit(1);
                });

                femtable.insert(
                    *fem_counter,
                    Femboy {
                        addr: femboy.1,
                        status: FemState::Idle,
                    },
                );
                println!("[\x1b[92mSUCC\x1b[0m] New femboy ID {fem_counter}");
                *fem_counter += 1;
            }

            tokio::spawn(async move { handle_femboy(femboy.0).await });
        }
    });

    let command_task = tokio::spawn(async move {
        let mut stdin = BufReader::new(io::stdin());
        let mut buf = String::new();
        loop {
            if let Err(_) = stdin.read_line(&mut buf).await {
                println!("[\x1b[91mERR\x1b[0m] Failed to read command");
                continue;
            }

            let command = buf.trim().to_lowercase();

            if command == "femboys" {
                let femtable = femtable.lock().unwrap_or_else(|_| {
                    process::exit(1);
                });

                println!("{femtable:?}");
            }

            buf.clear();
        }
    })
    .await;
}

async fn heartbeat(stream: Arc<Mutex<TcpStream>>) {
    let mut ticker = tokio::time::interval(Duration::from_secs(15));
    let config = bincode::config::standard();
    let hello = bincode::encode_to_vec(CMD::hello, config).unwrap_or_else(|_| {
        process::exit(1);
    });

    loop {
        ticker.tick().await;
        let mut stream = stream.lock().unwrap_or_else(|_| {
            panic!("Mutex gone :(");
        });

        stream.write_u16(hello.len() as u16).await.unwrap_or_else(|_| {
            panic!("Client disconnected :(");
        });
        
        stream.write_all(hello.as_slice()).await.unwrap_or_else(|_| {
            panic!("Client disconnected :(");
        });

        let size = stream.read_u16().await.unwrap_or_else(|_| {
            panic!("uh oh :(");
        });

        let mut buf = vec![0u8; size as usize];
        stream.read_exact(buf.as_mut_slice()).await.unwrap_or_else(|_| {
            panic!("uh oh :(");
        });

        match bincode::decode_from_slice::<CMD, _>(buf.as_slice(), config) {
            Ok(cmd) => {
                if let CMD::hello = cmd.0 {
                    println!("Got hello back");
                }
            },
            Err(_) => {
                println!("Did not get hello back");
            }
        };
    }
}

async fn handle_femboy(mut stream: TcpStream) {
    loop {
        stream.write_all(b"Hello World").await.unwrap();
        sleep(Duration::from_secs(1)).await;
    }
}
