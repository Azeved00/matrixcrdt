use std::fmt::{Formatter, Debug, Result};
use std::vec::Vec;
use std::cmp::{Ord, Ordering};
use hmac::Hmac;
use digest::{
    Digest, HashMarker, Mac,
    core_api::*,
    typenum::*,
    block_buffer::Eager,
    consts::U256,
};
use serde::{Serialize, Deserialize};
use super::Hash;


/// Node of the merkle dag
#[derive(Serialize, Deserialize, Clone)]
pub struct Node<O> 
    where O:Debug
{
    pub hash: Hash,
    pub parents: Vec<Hash>,
    pub data: O,
    pub layer: usize,
}

impl<O> Node<O>
    where O: Clone, O: Into<Vec<u8>>, O: Debug,
{
    /// Create a new merkle dag node,
    /// this will include creating the node's hash with digest `D` and key `key`
    /// both `data` and parents hashes (`parents`) will be hashed 
    pub fn new<D>(key: &Vec<u8>, data: &O, parents: &Vec<Hash>, layer: usize) -> Self 
        where 
            D: Digest,
            D: CoreProxy,
            D::Core: HashMarker + 
                UpdateCore + 
                FixedOutputCore + 
                BufferKindUser<BufferKind = Eager> + 
                Default + Clone,
            <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
            Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero, 
    {
        let mut mac = Hmac::<D>::new_from_slice(key)
                .expect("HMAC can take key of any size");

        let action_bytes: Vec<u8> = data.clone().into(); 
        mac.update(action_bytes.as_slice());

        for parent in parents {
            mac.update(&parent);
        }

        Node {
            data: data.clone(),
            parents: parents.clone(),
            hash: mac.finalize().into_bytes().to_vec(),
            layer,
        }
    }

    /// verifying the node, 
    /// takes a key and checks if (for the given digest)
    /// the hash of the node is correct
    pub fn verify<D>(&self, key: &Vec<u8>) ->  bool
        where 
            D: Digest,
            D: CoreProxy,
            D::Core: HashMarker + 
                UpdateCore + 
                FixedOutputCore + 
                BufferKindUser<BufferKind = Eager> + 
                Default + Clone,
            <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
            Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero, 
    {
        let mut mac = Hmac::<D>::new_from_slice(key)
                .expect("HMAC can take key of any size");

        let action_bytes: Vec<u8> = self.data.clone().into(); 
        mac.update(action_bytes.as_slice());

        for parent in &self.parents {
            mac.update(&parent);
        }

        let hash = mac.finalize().into_bytes().to_vec();
        return self.hash == hash;
    }
}

impl<O> Debug for Node<O>
    where O: Debug
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let formatted_parents: Vec<String> = self
            .parents
            .iter()
            .map(|vec| format!("{:?}", &vec[..vec.len().min(7)]))
            .collect();

        let hash = format!("{:?}",&&self.hash[..7]);
        f.debug_struct("Node")
            .field("hash", &hash)
            .field("parents",&formatted_parents)
            .field("data",  &self.data)
            .finish()
    }
}

impl<O> Ord for Node<O>
    where O:Debug
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.layer.cmp(&other.layer)
    }
}
impl<O> PartialOrd for Node<O> 
    where O:Debug
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<O> PartialEq for Node<O> 
    where O:Debug
{
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl<O> std::cmp::Eq for Node<O> 
    where O:Debug
{}

#[cfg(test)]
mod tests {
    use super::*;
    use sha3::Sha3_256;

    #[test]
    fn hash_correctness() {
        let key : Vec<u8> = "".to_string().into();
        let data: Vec<u8> = vec![];

        let mut mac = Hmac::<Sha3_256>::new_from_slice(&key)
                .expect("HMAC can take key of any size");

        let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, &vec![], &vec![vec![]], 0);


        let action_bytes: Vec<u8> = data; 
        mac.update(action_bytes.as_slice());
        let hash =  mac.finalize().into_bytes().to_vec();

        assert_eq!(hash, node.hash);
    }

    #[test]
    fn verification() {
        let key : Vec<u8> = "".to_string().into();
        let key2 : Vec<u8> = "123".to_string().into();
        let data: Vec<u8> = vec![];

        let mut node = Node::<Vec<u8>>::new::<Sha3_256>(&key, &data, &vec![vec![]], 0);

        assert!(node.verify::<Sha3_256>(&key));
        assert!(!node.verify::<Sha3_256>(&key2));
        
        node.data = vec![1,2];

        assert!(!node.verify::<Sha3_256>(&key));
    }
}
