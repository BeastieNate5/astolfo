use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum CMD {
    hello,
    attack(String),
    stop,
}

