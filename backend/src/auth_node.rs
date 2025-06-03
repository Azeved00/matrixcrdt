use std::fmt::Debug;
use std::hash;
use std::sync::Arc;
use serde::{
    self,
    Serialize, Deserialize,
};

use crate::Hash;
use crate::node::Node;

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthNode<O>
    where O: Clone, O: hash::Hash, O: Debug, O:PartialEq,
{
    pub node: Arc<Node<O>>,
    pub hash: Hash,
}
