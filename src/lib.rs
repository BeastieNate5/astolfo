use bincode::{Decode, Encode};

#[derive(PartialEq, Encode, Decode, Debug, Clone)]
pub enum BotState {
    Idle,
    Attacking(String),
    Dead,
}

