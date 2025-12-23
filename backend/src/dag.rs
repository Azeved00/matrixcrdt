use log::error;
use std::fmt::{Formatter, Debug, Result};
use std::vec::Vec;
use std::collections::{HashMap, VecDeque};
use dashmap::DashMap;
use std::cmp;
use std::io;
use std::sync::Arc;
use std::hash::Hash;
use serde::{Serialize, Deserialize};

use crate::node::MerkleNode;
use crate::QueryCursor;
use crate::auth_crdt::{Client};


/// The Merkle dag structure,
///
/// The structure makes use of 2 binary tree map structures.
///
/// Furthermore, the structure differentiates between a partial and a total Dag
/// a total dag is a dag such that for all nodes all parents are in the dag while
/// a partial dag is a dag in which the parents of some nodes are not inside the dag
#[derive(Clone)]
pub struct MerkleDag<O>
  where O: Clone, O: Debug, O:Hash, O:PartialEq,
          O:Serialize, O:for<'de> Deserialize<'de>
{
    dag: DashMap<u64, Arc<MerkleNode<O>>>,
    heads: DashMap<u64, Arc<MerkleNode<O>>>,
    topo: Vec<Arc<MerkleNode<O>>>,
    cursor: QueryCursor,
    partial: bool,
    top_layer: usize,
}

impl<O> Client<O> for MerkleDag<O>
  where O: Clone, O: Debug, O:Hash, O:PartialEq,
          O:Serialize, O:for<'de> Deserialize<'de>
{
    type Key = ();
    type NodeImpl = MerkleNode<O>;

    /// Create a new empty graph
    /// This is done by giving a key to perform the hashes
    fn new(key: (), client_id:u64) -> Self {
        Self{ 
            dag: DashMap::new(), 
            partial: false,
            top_layer: 0,
            heads: DashMap::new(), 
            topo: Vec::new(),
            cursor: QueryCursor::new(),
        }
    }

    /// Add a node to the network
    ///
    /// returns ok if the node is added correctly, in this case a cursor is returned as well 
    ///
    /// if the user provides a query cursor, it will be updated to include the new node,
    /// i.e. remove the new node's parents and add it to the cursor
    ///
    /// This method has temporal complexity: O(p. log n) where
    ///     p is number of parents of the node
    ///     n is the number of nodes of the graph
    fn insert(&mut self, data:O)
        -> io::Result<(Arc<MerkleNode<O>>)>
    {
        let parent_ids : Vec<u64> = self.cursor.heads.clone().into_iter().collect();
        let mut layer = 0usize;
        for parent_id in &parent_ids{
            let parent = self.get_node(parent_id);
            match parent {
                None => {
                    return Err(io::Error::new(io::ErrorKind::Other, "Not all parents of the node are in the Dag"));
                }
                Some(node) => {
                    layer = cmp::max(layer, node.layer);
                }
            }

            self.cursor.heads.remove(parent_id);
            self.heads.remove(parent_id);
        }

        layer += 1;
        self.top_layer = cmp::max(self.top_layer, layer);

        let node = MerkleNode::new(data, parent_ids, Some(layer), Some(self.topo.len()));

        let nrf = Arc::new(node.clone());
        self.dag.insert(nrf.id.clone(), Arc::clone(&nrf));
        self.topo.push(Arc::clone(&nrf));
        self.heads.insert(nrf.id.clone(), Arc::clone(&nrf));

        self.cursor.heads.insert(nrf.id.clone());
        
        return Ok(nrf);
    }

    /// Merge MerkleDag `dag` into `self`
    /// This means that all of the Nodes in `dag` which are not in `self`
    /// will be added to `self`
    ///
    /// Complefity $O(N+v)$ where 
    /// $N$ is the number of nodes and 
    /// $E$ is the number of conections between nodes
    fn merge(&mut self, dag:Self) -> io::Result<()> {
        todo!("merge is not yet implemented")
    }

    fn query(&self) -> Vec<O>
    {
        let mut cursor = self.cursor;
        let mut index = self.topo.len()-1;
        let size = self.topo.len() - cursor.index;
        let mut vis :Vec<bool> = vec![false; size.try_into().unwrap()];
        let mut res = Vec::<O>::new();

        for head_hash in &cursor.heads {
            let head = self.get_node(&head_hash).unwrap();
            if head.index > cursor.index {
                vis[head.index - cursor.index] = true;
            }
        }

        while index >= cursor.index {
            let top = &self.topo[index];

            for parent_hash in &top.parents {
                let parent = self.get_node(&parent_hash).unwrap();
                if parent.index < cursor.index {
                    continue
                }
                else if vis[top.index - cursor.index]{
                    vis[parent.index - cursor.index] = true;
                } 
            }
            
            if !vis[top.index - cursor.index]{
                res.push(top.data.clone());
            } 
            if index == 0 {
                break;
            }
            index -= 1;
        }

        let mut cursor = QueryCursor::new();
        cursor.set_heads(self.get_heads());
        cursor.index = self.topo.len();

        return res.into_iter().rev().collect()
    }
}


impl<O> MerkleDag<O> 
    where O: Clone, O: Debug, O:Hash, O:PartialEq,
          O:Serialize, O:for<'de> Deserialize<'de>
{
    //------------------------- HELPER FUNCTIONS -----------------------------

    /// Returns the length of the dag,
    /// i.e. the number of nodes in it
    pub fn len(&self) -> usize {
        return self.dag.len();
    }

    pub fn get_top_layer(&self) -> usize {
        return self.top_layer;
    }

    pub(crate) fn get_node(&self, id: &u64) -> Option<Arc<MerkleNode<O>>> {
        match self.dag.get(id){
            Some(x) => {
                let n:Arc<MerkleNode<O>> = (*x.value()).clone();
                Some(n)
            },
            None => None
        }
    }

    /// get the head nodes of the dag
    pub fn get_heads(&self) -> Vec<u64> {
        return self.heads.iter().map(|r| r.key().clone()).collect();
    }

    //------------------------- SPEC IMPLEMENTATION -----------------------------
    pub fn insert_node(&mut self, node:MerkleNode<O>, opt_cursor: Option<QueryCursor>)
        -> io::Result<(Arc<MerkleNode<O>>, QueryCursor)>
    {
        let mut cursor = match opt_cursor {
            None => QueryCursor::new(),
            Some(c) => c,
        };

        let mut layer = 0usize; 
        for parent_hash in &node.parents {
            let parent = self.dag.get(parent_hash);
            match parent {
                None => {
                    return Err(io::Error::new(
                            io::ErrorKind::Other, 
                            "Not all parents of the node are in the Dag"));
                }
                Some(node) => {
                    layer = cmp::max(layer, node.layer);
                }
            }
            self.heads.remove(parent_hash);
            cursor.heads.remove(parent_hash);
        }

        layer += 1;
        self.top_layer = cmp::max(self.top_layer, layer);

        let mut nn = node.clone();
        nn.index = self.topo.len();
        nn.layer = layer;

        let nrf = Arc::new(nn);
        self.dag.insert(nrf.id.clone(), Arc::clone(&nrf));
        self.topo.push(Arc::clone(&nrf));
        self.heads.insert(nrf.id.clone(), Arc::clone(&nrf));

        cursor.heads.insert(nrf.id);
        return Ok((nrf, cursor));
    }
    
 
    /// Return a valid linearization of the Dag,
    /// i.e. a topological sort of the Dag
    ///
    /// Khan's topological sort algorithm was used to calculate the answer
    /// furthermore this function has complexity:
    /// - Time: $O(V*log(v) + E*log(v))$
    /// - Space: $O(V)$
    /// where $V$ is number of vertices of the graph and 
    /// $E$ the number of edges of the graph
    pub fn linearize(&self) -> Vec<MerkleNode<O>> {
        self.topo.iter()
            .map(|arc| MerkleNode::clone(&*arc))
            .collect()
    }
}


impl<O>Debug for MerkleDag<O> 
    where O: Clone, O:Debug, O:Hash, O:PartialEq,
          O:Serialize, O:for<'de> Deserialize<'de>
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("Merkle Dag")
            .field("length", &self.dag.len())
            .field("partial", &self.partial)
            .field("heads",  &self.heads)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialization() {
        let dag = MerkleDag::<u8>::new((),0);

        assert_eq!(dag.get_top_layer(), 0);
        assert_eq!(dag.len(), 0);
    }
    
    #[test]
    fn node_insertion(){
        let mut dag = MerkleDag::<u8>::new((),0);
        let node1 = MerkleNode::new(1, vec![], None, None);

        assert!(dag.insert_node(node1, None).is_ok(), 
            "Node was not added correctly");
        assert_eq!(dag.len(), 1);

        let node2 = MerkleNode::new(2, vec![3], None, None);
        assert!(dag.insert_node(node2, None).is_err(), 
            "Node whose parents were not in the dag was added");
        assert_eq!(dag.len(), 1);
    }

    #[test]
    fn querying_loose_heads(){
        let mut dag = MerkleDag::<u8>::new();

        let mut c = QueryCursor::new();

        for i in 0..10 as usize {
            let node = MerkleNode::new(i as u8, vec![], None, None);
            (_, c) = dag.insert_node(node, Some(c)).unwrap();
            assert_eq!(c.heads.len(), i+1, 
                "the cursor should include all heads of the dag");
        }

        let (changes, cursor) = dag.query(None);
        let len1 = changes.len();
        assert_eq!(len1, dag.len());
            
        for i in 10..20 as usize {
            let node = MerkleNode::new(i as u8, vec![], None, None);
            dag.insert_node(node, None).unwrap();
        }

        let (changes, _c) = dag.query(None);
        assert_eq!(changes.len(), dag.len(), 
            "Querying without cursor, outputs the full dag");

        println!("{:}", cursor.index);
        let (changes, cursor2) = dag.query(Some(cursor));
        println!("{:}", cursor2.index);
        assert_eq!(changes.len() + len1, dag.len(), 
            "Querying with cursor outputs the new nodes");

        let (changes, _c) = dag.query(Some(cursor2));
        assert_eq!(changes.len() , 0 , 
            "Querying twice in a row (with cursor) outputs empty changes array");
    }
    
    #[test]
    fn query_branches_with_overlaping_history() {
        let mut dag = MerkleDag::<u8>::new();
        let mut cursor = QueryCursor::new(); 
        (_, cursor) = dag.insert(0, Some(cursor)).unwrap();

        for i in 0..5 as usize {
            (_, cursor) = dag.insert(i as u8, Some(cursor)).unwrap();
        }

        let (_, cursor1) = dag.insert(7, Some(cursor.clone())).unwrap();
        println!("dag heads {:?}", dag.get_heads());
        println!("cursor {:?}", cursor1);
        println!("");

        let (_, cursor2) = dag.insert(8, Some(cursor.clone())).unwrap();
        println!("dag heads {:?}", dag.get_heads());
        println!("cursor {:?}", cursor2);
        println!("");

        let (res1, _)= dag.query(Some(cursor1));
        println!("result {:?}", res1);
        println!("");

        let (res2, _)= dag.query(Some(cursor2));
        println!("result {:?}", res2);

        assert!(res1.len() == 1, "The result of the query should be an array with only 1 element(the new node)");
        assert!(res2.len() == 1, "The result of the query should be an array with only 1 element(the new node)");
    }

    #[test]
    fn cursor_loose_heads() {
        let mut dag = MerkleDag::<u8>::new();

        let (node, cursor) = dag.insert(200, None).unwrap();
        let mut loose = cursor.clone();
        let mut otherids: Vec<u64> = vec![];

        assert!(cursor.contains(&node.id), 
            "the first cursor should have the first node added");

        for i in 1..11 as usize {
            let node = MerkleNode::new(i as u8,vec![node.id.clone()], None, None);
            let nid = node.id.clone();
            (_, loose) = dag.insert_node(node, Some(loose)).unwrap();

            println!("heads {:?}", dag.get_heads());

            assert!(loose.contains(&nid), 
                "the new cursor should include the new node");

            for id in &otherids {
                assert!(loose.contains(&id), 
                    "the new cursor should contain the other heads");
            }
            otherids.push(nid);
        }
    }

    #[test]
    // when adding to existing branches only the head should be there
    fn query_cursor_single_branch() {
        let mut dag = MerkleDag::<u8>::new();

        let (node, mut cursor) = dag.insert(0u8, None).unwrap();
        assert!(cursor.contains(&node.id), 
            "the first cursor should have the first node added");

        for i in 1..11 as usize {
            (_, cursor) = dag.insert(i as u8, Some(cursor)).unwrap();

            println!("heads {:?}", dag.get_heads());
            println!("cursor {:?}", cursor);

            assert_eq!(cursor.heads.len(), 1, 
                "the new cursor should ONLY include the new node");
        }
    }
}
