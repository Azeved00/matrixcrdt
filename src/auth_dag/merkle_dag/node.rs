use std::fmt::{Formatter, Debug, Result};
use std::vec::Vec;
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
}

impl<O> Node<O>
    where O: Clone, O: Into<Vec<u8>>, O: Debug,
{
    /// Create a new merkle dag node,
    /// this will include creating the node's hash with digest `D` and key `key`
    /// both `data` and parents hashes (`parents`) will be hashed 
    pub fn new<D>(key: Vec<u8>, data: &O, parents: &Vec<Hash>) -> Self 
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
        let mut mac = Hmac::<D>::new_from_slice(&key)
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
        }
    }

    /// verifying the node, 
    /// takes a key and checks if (for the given digest)
    /// the hash of the node is correct
    pub fn verify<D>(&self, key: Vec<u8>) ->  bool
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
        let mut mac = Hmac::<D>::new_from_slice(&key)
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
