use sha3::Sha3_256;
use std::fmt;

pub mod node;
pub mod dag;
pub mod auth;

pub mod common {
    pub mod logger;
    pub mod message;
}

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

impl fmt::Debug for QueryCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueryCursor")
            .field("set", &self.set)
            .finish()
    }
}
