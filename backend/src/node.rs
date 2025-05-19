use std::fmt::{Formatter, Debug, Result};
use std::vec::Vec;
use std::cmp::{Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;


#[derive(Clone)]
pub struct Node<O> 
    where O:Clone, O:Debug, O:Hash
{
    pub parents: Vec<u64>,
    pub data: O,
    pub id: u64,
    pub layer: usize,
    pub index: usize,
}

impl<O> Node<O>
    where O:Clone, O:Debug, O:Hash
{
    pub(crate) fn new(data: O, parents: Vec<u64>, olayer: Option<usize>, oindex:Option<usize>) -> Node<O>{
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        parents.hash(&mut hasher);
        let hash = hasher.finish();

        Node{
            id:hash,
            data,
            parents,
            layer: match olayer{ None => 0, Some(l) => l},
            index: match oindex{ None => 0, Some(i) => i},
        }
    }
}

impl<O> Hash for Node<O> 
    where O:Clone, O:Debug, O:Hash
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.id);
    }
}

impl<O> Debug for Node<O>
    where O:Clone, O:Debug, O:Hash
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("layer", &self.layer)
            .field("parents",&self.parents)
            .field("data",  &self.data)
            .finish()
    }
}


impl<O> Ord for Node<O>
    where O:Clone, O:Debug, O:Hash
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.layer.cmp(&other.layer)
    }
}
impl<O> PartialOrd for Node<O> 
    where O:Clone, O:Debug, O:Hash
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<O> PartialEq for Node<O> 
    where O:Clone, O:Debug, O:Hash
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<O> std::cmp::Eq for Node<O> 
    where O:Clone, O:Debug, O:Hash
{}

