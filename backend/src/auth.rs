use std::fmt::{self, Formatter, Debug};
use std::io;
use std::vec::Vec;
use core::marker::PhantomData;
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
    pub fn gen_node(&self, data: O, opt_cursor: Option<QueryCursor>) -> Node<O> {
        let parents = match opt_cursor{
            None => self.dag.get_heads(),
            Some(cursor) => cursor.set.into_iter().collect(),
        };
        let layer = self.dag.get_top_layer();
        let node = Node::new::<D>(&self.key, &data, &parents, layer + 1);
        node
    }

    /// Insert a new node into the authenticated merkle dag,
    pub fn add_node(&mut self, node: Node<O>, opt_cursor: Option<QueryCursor>) -> io::Result<QueryCursor> {
        if !node.verify::<D>(&self.key) {
            return Err(io::Error::new(io::ErrorKind::Other, "Hash of node is not properly formed."));
        }
         return self.dag.add_node(node.clone(), opt_cursor);
    }

    /// Verify the authenticated merkle dag
    ///
    /// This function verifies the DAG, this means that
    /// 1. the hashes in each of the nodes are correct and
    /// 2. all parents of all nodes are inside the graph
    /// if these conditions are met then `true` is returned
    pub fn verify(&self) -> bool {
        self.dag.dag.values().all(|node| 
            node.verify::<D>(&self.key) &&
            node.parents.iter().all(|parent_hash| {
                if let Some(_) = self.dag.dag.get(parent_hash) {
                    true
                } else {
                    false
                }
        }))
    }

    /// Calculate the union between `self` and `d`
    ///
    /// This function first check if d is fully contained by d
    /// if yes then nothing is done
    /// otherwise union of dags is performed
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
        let mut dag = AuthMerkleDag::<Sha3_256,Vec<u8>>::new(password);
        
        let node = dag.gen_node(vec![1], None);
        assert_eq!(node.parents.len(), 0);
        assert_eq!(dag.len(), 0);
        assert!(dag.verify());

        let mut node_mod = node.clone();
        node_mod.hash = "".into();

        let res = dag.add_node(node_mod, None);
        assert!(res.is_err(), "Insertion should fail if node has incorrect hash");
        assert_eq!(dag.len(), 0);
        assert!(dag.verify());

        let res = dag.add_node(node, None);
        assert!(res.is_ok());
        assert_eq!(dag.len(), 1);
        assert!(dag.verify());
    }
}
