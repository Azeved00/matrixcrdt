use std::fmt::{Formatter, Debug, self};
use std::vec::Vec;
use std::cmp::{Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use serde::{
    self,
    Serialize, Deserialize,
};

use serde_with::{serde_as, DisplayFromStr};
use crate::auth_crdt::Node;

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct MerkleNode<O> 
    where O:Clone, O:Debug, O:Hash, O:PartialEq,
{
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub parents: Vec<u64>,
    pub data: O,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub layer: usize,

    #[serde(skip_serializing)]
    #[serde(default)]
    pub index: usize,

    #[serde(skip_serializing)]
    #[serde(default)]
    pub id: u64,
}

impl<O> Node<O> for MerkleNode<O>
    where O:Clone, O:Debug, O:Hash, O:PartialEq,
{
    type Key = ();
    type Hash = u64;

    fn new(_: (), data: O, parents: Vec<u64>) -> Self{
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        parents.hash(&mut hasher);
        let hash = hasher.finish();

        Self {
            id:hash,
            data,
            parents,
            layer: 0,
            index: 0,
        }
    }
}

impl<O> Hash for MerkleNode<O> 
    where O:Clone, O:Debug, O:Hash, O:PartialEq,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.id);
    }
}

impl<O> Debug for MerkleNode<O>
    where O:Clone, O:Debug, O:Hash, O:PartialEq,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("layer", &self.layer)
            .field("parents",&self.parents)
            .field("data",  &self.data)
            .finish()
    }
}


impl<O> Ord for MerkleNode<O>
    where O:Clone, O:Debug, O:Hash, O:PartialEq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.layer.cmp(&other.layer)
    }
}
impl<O> PartialOrd for MerkleNode<O> 
    where O:Clone, O:Debug, O:Hash, O:PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<O> PartialEq for MerkleNode<O> 
    where O:Clone, O:Debug, O:Hash, O:PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.parents.len() != other.parents.len() {
            return false;
        }
        let mut a_sorted = self.parents.to_vec();
        let mut b_sorted = other.parents.to_vec();
        a_sorted.sort_unstable();
        b_sorted.sort_unstable();

        self.data == other.data && a_sorted == b_sorted
    }
}

impl<O> std::cmp::Eq for MerkleNode<O> 
    where O:Clone, O:Debug, O:Hash, O:PartialEq, 
{}

