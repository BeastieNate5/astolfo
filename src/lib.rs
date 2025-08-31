use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone)]
pub enum FemState {
    Idle,
    Attacking(String),
    Dead,
}

#[derive(Encode, Decode)]
pub enum CMD {
    hello,
    attack(String),
    stop,
}
