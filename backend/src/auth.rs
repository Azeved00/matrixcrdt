use std::fmt::{Formatter, Debug, Result};
use std::vec::Vec;
use core::marker::PhantomData;
use std::collections::HashSet;
use digest::{
    Digest, HashMarker,
    core_api::*,
    typenum::*,
    block_buffer::Eager,
    consts::U256,
};
use super::{
    Hash, QueryCursor,
    node::Node,
    dag::MerkleDag,
};

#[derive(Clone)]
pub struct AuthMerkleDag<D:Digest, O> 
  where O: Clone, O: Into<Vec<u8>>, O: Debug,
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
}


impl<D: Digest, O> AuthMerkleDag<D, O> 
    where O: Clone, O: Into<Vec<u8>>, O: Debug,
        D: CoreProxy,
        D::Core: HashMarker + 
            UpdateCore + 
            FixedOutputCore + 
            BufferKindUser<BufferKind = Eager> + 
            Default + Clone,
        <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
        Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero, 
{
    /// Create a new Authenticated Merkle Dag 
    /// `key` will be used to hash the nodes
    pub fn new(key: Vec<u8>) -> Self {
        Self {
            dag: MerkleDag::new(), 
            hasher: PhantomData::<D>,
            key,
        }
    }
    pub fn len(&self) -> usize {
        return self.dag.len();
    }

    /// Generate a node from the current merkle dag
    ///
    /// it's parents will be the heads of the merkle dag 
    /// or, in the case a cursor is given, the heads represented by the cursor
    ///
    /// this function also returns a cursor which simply represents the returned node
    pub fn gen_node(&self, data: O, opt_cursor: Option<QueryCursor>) -> (Node<O>, QueryCursor) {
        let parents = match opt_cursor{
            None => self.dag.get_heads(),
            Some(cursor) => cursor.set.into_iter().collect(),
        };
        let layer = self.dag.get_top_layer();
        let node = Node::new::<D>(self.key.clone(), &data, &parents, layer + 1);
        let cursor = QueryCursor {set: HashSet::from([node.hash.clone()]) };
        (node, cursor)
    }

    /// Insert a new node into the authenticated merkle dag,
    pub fn add_node(&mut self, node: Node<O>){
        self.dag.add_node(node.clone());
    }

    /// Verify the authenticated merkle dag
    pub fn verify(&self) -> bool {
        self.dag.verify::<D>(&self.key)
    }

    /// Calculate the union between `self` and `d`
    ///
    /// This function first check if d is fully contained by d
    /// if yes then nothing is done
    /// otherwise union of dags is performed
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

    /// calculate the linearization of the Authenticated Merkle Dag
    pub fn linearize(&self) -> Vec<Node<O>> {
        self.dag.linearize()
    }

    ///Retreive a copy of the dag
    pub fn get_dag(&self) -> MerkleDag<O> {
        self.dag.clone()
    }

    /// Get the node from the dag with the specified Hash
    ///
    /// if there is no node with the given hash, `None` is returned
    pub fn get_node(&self, hash: &Hash) -> Option<&Node<O>> {
        self.dag.get_node(hash)
    }

    pub fn query(&self,opt_cursor: Option<QueryCursor>) -> (Vec<O>, QueryCursor)
    {
        self.dag.query(opt_cursor)
    }
}

impl<D: Digest,O>Debug for AuthMerkleDag<D, O> 
    where O: Clone, O: Into<Vec<u8>>, O:Debug,
        D: CoreProxy,
        D::Core: HashMarker + 
            UpdateCore + 
            FixedOutputCore + 
            BufferKindUser<BufferKind = Eager> + 
            Default + Clone,
        <D::Core as BlockSizeUser>::BlockSize: IsLess<U256>,
        Le<<D::Core as BlockSizeUser>::BlockSize, U256>: NonZero, 
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("Authenticated Merkle Dag")
            .field("dag", &self.dag)
            .finish()
    }
}
