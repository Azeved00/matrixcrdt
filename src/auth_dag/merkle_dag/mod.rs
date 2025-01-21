pub mod node;
pub mod dag;
pub mod auth;

use std::collections::HashSet;

pub type Hash = Vec<u8>;

#[derive(Clone, Default)]
pub struct QueryCursor {
    pub (crate) set: HashSet<Hash>
}
