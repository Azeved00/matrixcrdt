use std::fmt::{Formatter, Debug, self};
use std::vec::Vec;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use serde::{
    self,
    Serialize, Deserialize,
};

use serde_with::{serde_as, DisplayFromStr};
use crate::traits::Node;

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct MerkleNode<O> 
    where O:Clone, O:Debug, O:Hash, O:Serialize
{
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub parents: Vec<u64>,

    #[serde(skip_serializing)]
    #[serde(default)]
    pub id: u64,

    pub data: O,
}

impl<O> Node<O> for MerkleNode<O>
    where O:Clone, O:Debug, O:Hash, O:Serialize
{
    type Key = String;
    type Hash = u64;

    fn new(_: &String, data: O, parents: Vec<u64>) -> Self{
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        parents.hash(&mut hasher);
        let hash = hasher.finish();

        Self {
            id:hash,
            data,
            parents,
        }
    }

    fn verify (&self, _:&String) -> bool {
        let mut hasher = DefaultHasher::new();
        self.data.hash(&mut hasher);
        self.parents.hash(&mut hasher);
        let hash = hasher.finish();

        return self.id == hash;
    }

    fn get_id(&self) -> Self::Hash {self.id}
    fn get_data(&self) -> O {self.data.clone()}
    fn get_parents(&self) -> Vec<Self::Hash> {self.parents.clone()}
}

impl<O> Hash for MerkleNode<O> 
    where O:Clone, O:Debug, O:Hash, O:Serialize
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.id);
    }
}

impl<O> Debug for MerkleNode<O>
    where O:Clone, O:Debug, O:Hash, O:Serialize
{
   fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("parents",&self.parents)
            .field("data",  &self.data)
            .finish()
    }
}
