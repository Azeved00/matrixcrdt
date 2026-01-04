pub mod cursor;
pub use cursor::QueryCursor;

pub mod traits;
pub mod node {
    pub mod auth_node;
    pub mod node;

    pub use self::node::MerkleNode;
    pub use self::auth_node::AuthNode;
}
pub mod dag;
pub mod matrix_dag;

pub mod common {
#[cfg(feature = "bench")]
    pub mod logger;
    pub mod message;
}


pub type Hash = Vec<u8>;
pub type MerkleDag = dag::MerkleDag<Vec<u8>, node::MerkleNode<Vec<u8>>>;
pub type AuthDag = dag::MerkleDag<Vec<u8>, node::AuthNode<Vec<u8>>>;

