use std::fmt::{self, Formatter, Debug};
use std::hash;
use std::collections::HashMap;
use std::sync::Arc;
use std::io;
use std::vec::Vec;
use core::marker::PhantomData;
use hmac::Hmac;
use digest::{
    Digest, HashMarker, Mac,
    core_api::*,
    typenum::*,
    block_buffer::Eager,
    consts::U256,
};
use serde::{Serialize, Deserialize};
use serde_json;

use crate::{
    Hash, QueryCursor,
    dag::MerkleDag,
    node::Node,
    auth_node::AuthNode,
};

#[derive(Clone)]
pub struct AuthMerkleDag<D:Digest, O> 
    where O: Clone, O: hash::Hash, O: Debug, O:PartialEq,
          O:Serialize, O:for<'de> Deserialize<'de>,
        D: CoreProxy,
        D::Core: HashMarker + 
            UpdateCore + 
            FixedOutputCore + 
            BufferKindUser<BufferKind = Eager> + 
            Default + Clone,
        <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
        Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero, 
{
    hasher: PhantomData<D>,
    key: Vec<u8>,
    dag: MerkleDag<O>,

    hashes: HashMap<u64, Hash>,
}

impl<D: Digest, O> AuthMerkleDag<D, O> 
    where O: Clone, O: hash::Hash, O: Debug, O:PartialEq,
          O:Serialize, O:for<'de> Deserialize<'de>,
        D: CoreProxy,
        D::Core: HashMarker + 
            UpdateCore + 
            FixedOutputCore + 
            BufferKindUser<BufferKind = Eager> + 
            Default + Clone,
        <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
        Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero, 
{
    //===================HELPER METHODS=========================================
    pub fn len(&self) -> usize {
        return self.dag.len();
    }

    ///Retreive a copy of the dag
    pub fn get_dag(&self) -> MerkleDag<O> {
        self.dag.clone()
    }

    /// Get the node from the dag with the specified Hash
    ///
    /// if there is no node with the given hash, `None` is returned
    fn get_node(&self, id: &u64) -> Option<Arc<Node<O>>> {
        self.dag.get_node(id)
    }

    fn get_auth_node(&self, id: &u64) -> Option<AuthNode<O>> {
        match self.dag.get_node(id) {
            Some(node) => Some(AuthNode {
                node,
                hash: self.hashes.get(id).unwrap().clone(),
            }),
            None => None
        }
    }
    
    pub fn calc_hash(&self, data:O, parents:Vec<u64>) -> io::Result<Vec<u8>> {
        let mut mac = Hmac::<D>::new_from_slice(&self.key)
                .expect("HMAC can take key of any size");

        for parent_id in parents {
            match self.hashes.get(&parent_id) {
                None => {
                    return Err(io::Error::new(
                            io::ErrorKind::Other, 
                            "Not all parents of the node are in the Dag"));
                }
                Some(parent_hash) => {
                    mac.update(&parent_hash);
                }
            }
        }

        let bytes = serde_json::to_vec(&data).expect("serialization failed"); 
        mac.update(&bytes);

        let hash = mac.finalize().into_bytes().to_vec();
        Ok(hash)
    }

    //=======================SPEC IMPLEMENTATION==================================
    /// Create a new Authenticated Merkle Dag 
    /// `key` will be used to hash the nodes
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            dag: MerkleDag::new(), 
            hasher: PhantomData::<D>,
            key,
            hashes: HashMap::new(),
        }
    }

    /// Insert a new node into the authenticated merkle dag,
    pub fn insert(&mut self, data:O, opt_cursor: Option<QueryCursor>) -> io::Result<(AuthNode<O>, QueryCursor)> {
        let parents = match opt_cursor.clone(){
            None => self.dag.get_heads(),
            Some(cursor) => cursor.heads.into_iter().collect(),
        };
        let hash = self.calc_hash(data.clone(), parents)?;

        return match self.dag.insert(data, opt_cursor) {
            Err(e) => Err(e),
            Ok((node, c)) => {
                self.hashes.insert(node.id, hash.clone());
                let anode = AuthNode {
                    node,
                    hash,
                };

                return Ok((anode, c));
            }
        };
    }
    pub fn insert_node(&mut self, auth_node:AuthNode<O>, opt_cursor: Option<QueryCursor>) -> io::Result<(AuthNode<O>, QueryCursor)> {
        let node = (*auth_node.node).clone();
        let hash = self.calc_hash(node.data.clone(), node.parents.clone())?;
        if auth_node.hash != hash {
            return Err(io::Error::new(
                    io::ErrorKind::Other, 
                    "Invalid Hash"));
        }

        return match self.dag.insert_node(node.clone(), opt_cursor) {
            Err(e) => Err(e),
            Ok((nref, c)) => {
                self.hashes.insert(node.id, hash.clone());

                let anode = AuthNode {
                    node: nref,
                    hash,
                };

                return Ok((anode, c));
            }
        };
    }

    /// Verify the authenticated merkle dag
    ///
    /// This function verifies the DAG, this means that
    /// 1. the hashes in each of the nodes are correct and
    /// 2. all parents of all nodes are inside the graph
    /// if these conditions are met then `true` is returned
    pub fn verify(&self) -> bool {
        self.hashes.clone().into_iter().all(
            |(node_id, hash)| {
                let onode = self.get_node(&node_id);
                match onode {
                    None => false,
                    Some(node) => {
                        (match self.calc_hash(node.data.clone(), node.parents.clone()) {
                            Ok(h) => hash == h,
                            Err(_) => false,
                        })&&
                        node.parents.iter().all(|parent| {
                            if let Some(_) = self.hashes.get(parent) {
                                true
                            } else {
                                false
                            }
                        })
                    }
                }
            })
    }

    /// Calculate the union between `self` and `d`
    ///
    /// This function first check if d is fully contained by d
    /// if yes then nothing is done
    /// otherwise union of dags is performed
    /*
    #[deprecated]
    pub fn union(&mut self,d : MerkleDag<O>) -> bool{
        let all_inside = d.heads.keys().all(|leaf| {
            let x = self.dag.get_node(leaf);
            match x {
                None => false,
                Some(_) => true,
            }

        });
        if !all_inside {
            self.dag.union(d);
            return true;
        }
        return false;
    }
    */

    /// calculate the linearization of the Authenticated Merkle Dag
    pub fn linearize(&self) -> Vec<Node<O>> {
        self.dag.linearize()
    }



    pub fn query(&self,opt_cursor: Option<QueryCursor>) -> (Vec<O>, QueryCursor)
    {
        self.dag.query(opt_cursor)
    }
}

impl<D: Digest,O>Debug for AuthMerkleDag<D, O> 
    where O: Clone, O: hash::Hash, O:Debug, O:PartialEq,
          O:Serialize, O:for<'de> Deserialize<'de>,
        D: CoreProxy,
        D::Core: HashMarker + 
            UpdateCore + 
            FixedOutputCore + 
            BufferKindUser<BufferKind = Eager> + 
            Default + Clone,
        <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
        Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero, 
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Authenticated Merkle Dag")
            .field("dag", &self.dag)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha3::Sha3_256;

    #[test]
    fn initialization() {
        let password : Vec<u8> = "random password".to_string().into();
        let dag = AuthMerkleDag::<Sha3_256,Vec<u8>>::new(password);

        assert_eq!(dag.len(), 0);
        assert!(dag.verify());
    }
    
    #[test]
    fn generation_insertion() {
        let password : Vec<u8> = "random password".to_string().into();
        let dag = AuthMerkleDag::<Sha3_256,Vec<u8>>::new(password);
        

        //assert!(res.is_err(), "Insertion should fail if node has incorrect hash");
    }

    /*#[test]
    fn verifying(){
        let password : Vec<u8> = "random password".to_string().into();
        let mut dag = AuthMerkleDag::<Sha3_256,Vec<u8>>::new(password.clone());

        for i in 0..10 as u8 {
            let node = dag.gen_node(vec![i], None);
            dag.add_node(node, None).unwrap();
        }

        let node42 = dag.gen_node(vec![42], None);
        let node42_hash = node42.hash.clone();
        dag.add_node(node42, None).unwrap();

        let node76 = dag.gen_node(vec![76], None);
        //let node76_hash = node76.hash.clone();
        dag.add_node(node76, None).unwrap();

        assert!(dag.verify(), "Verification should be fine");

        let node42 = dag.dag.dag.remove(&node42_hash).unwrap();
        assert!(!dag.verify(), "Verification should fail if a node's parent is not inside the dag");
        dag.add_node((*node42).clone(), None).unwrap();
        assert!(dag.verify(), "Re-inserting the node should restore the verifiability");

        // testing if an unverified node is inside the dag is not necessary
        // this is assured by the compiler : 
        // (i) you cant insert an invalid node and 
        // (ii) you cant modify a node you get from the get method
        /*let mut node76 : Node<Vec<u8>> = dag.dag.dag.remove(&node76_hash).unwrap().clone();
        node76.data = vec![0];
        assert!(!node76.verify::<Sha3_256>(&password));
        dag.add_node(node76, None).unwrap();
        assert!(!dag.verify(), "Verification should fail if a node's verification fails");
        */
    }*/
}
