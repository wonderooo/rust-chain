use std::fmt::Display;

use serde::{de::DeserializeOwned, Serialize};

pub mod block;
pub mod blockchain;
pub mod pow;
pub mod transaction;

pub trait Blockchainable: Serialize + DeserializeOwned + Display {
    fn genesis_data() -> Self;
}
