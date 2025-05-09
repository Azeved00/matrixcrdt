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
    pub (crate) forks: HashSet<Hash>
}

impl QueryCursor {
    pub fn new() -> Self {
        QueryCursor {
            heads: HashSet::new(),
            forks: HashSet::new(),
        }
    }

    pub fn is_head(&self, hash: &Hash) -> bool {
        self.heads.contains(hash)
    }

    pub fn is_fork(&self, hash: &Hash) -> bool {
        self.forks.contains(hash)
    }

    /// Weather a hash is referenced in this cursor
    /// either as a head node or as a fork node
    pub fn contains(&self, hash: &Hash) -> bool {
        self.is_head(hash) || self.is_fork(hash)
    }
}

impl fmt::Debug for QueryCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueryCursor")
            .field("heads", &self.heads)
            .field("forks", &self.forks)
            .finish()
    }
}
