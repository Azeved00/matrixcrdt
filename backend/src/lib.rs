use sha3::Sha3_256;
use std::fmt;

pub mod node;
pub mod dag;
pub mod auth;

pub mod common {
#[cfg(feature = "bench")]
    pub mod logger;
    pub mod message;
}

use std::collections::HashSet;

pub type Hash = Vec<u8>;
pub type AuthDag = auth::AuthMerkleDag<Sha3_256, Vec<u8>>;

#[derive(Clone, Default)]
pub struct QueryCursor {
    pub (crate) heads: HashSet<Hash>,
    pub (crate) index: usize,
}

impl QueryCursor {
    pub fn new() -> Self {
        QueryCursor {
            heads: HashSet::new(),
            index: 0,
        }
    }

    /// Weather a hash is referenced in this cursor
    /// either as a head node or as a fork node
    pub fn contains(&self, hash: &Hash) -> bool {
        self.heads.contains(hash)
    }
}

impl fmt::Debug for QueryCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueryCursor")
            .field("heads", &self.heads)
            .field("index", &self.index)
            .finish()
    }
}
