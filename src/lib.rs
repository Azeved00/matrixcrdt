use serde::{Deserialize, Serialize};

pub mod tui;
pub mod store_crdt;


#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum StoreCommand{
    Add(u64, u64),
    Remove(u64, u64),
    Delete(u64),
}

pub type Hash = Vec<u8>;
