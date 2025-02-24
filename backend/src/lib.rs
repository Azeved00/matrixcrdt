use sha3::Sha3_256;

pub mod node;
pub mod dag;
pub mod auth;

use std::collections::HashSet;

pub type Hash = Vec<u8>;
pub type AuthDag = auth::AuthMerkleDag<Sha3_256, Vec<u8>>;

#[derive(Clone, Default)]
pub struct QueryCursor {
    pub (crate) set: HashSet<Hash>
}

impl QueryCursor {
    pub fn new() -> Self {
        QueryCursor {
            set: HashSet::new(),
        }
    }

    pub fn contains(&self, hash: &Hash) -> bool {
        self.set.contains(hash)
    }
}

