use sha3::Sha3_256;
use std::fmt;

pub mod node;
pub mod dag;
pub mod auth_node;
pub mod auth_dag;

pub mod common {
#[cfg(feature = "bench")]
    pub mod logger;
    pub mod message;
}

use std::collections::HashSet;

pub type Hash = Vec<u8>;
pub type AuthDag = auth_dag::AuthMerkleDag<Sha3_256, Vec<u8>>;

#[derive(Clone, Default)]
pub struct QueryCursor {
    pub (crate) heads: HashSet<u64>,
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
    pub fn contains(&self, id: &u64) -> bool {
        self.heads.contains(id)
    }
    
    pub fn set_heads(&mut self, heads: Vec<u64>) {
        self.heads = HashSet::from_iter(heads);
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
