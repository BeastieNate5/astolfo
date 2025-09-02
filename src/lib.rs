use bincode::{Decode, Encode};

#[derive(PartialEq, Encode, Decode, Debug, Clone)]
pub enum FemState {
    Idle,
    Attacking(String),
    Dead,
}

