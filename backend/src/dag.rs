use log::error;
use std::fmt::{Formatter, Debug, Result};
use std::vec::Vec;
use std::collections::BTreeMap;
use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::cmp;
use std::io;
use std::sync::Arc;
use std::hash::Hash;
use serde::{Serialize, Deserialize};

use crate::node::Node;
use crate::QueryCursor;


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
    dag: BTreeMap<u64, Arc<Node<O>>>,
    heads: BTreeMap<u64, Arc<Node<O>>>,
    topo: Vec<Arc<Node<O>>>,
    partial: bool,
    top_layer: usize,
}


impl<O> MerkleDag<O> 
    where O: Clone, O: Debug, O:Hash, O:PartialEq,
          O:Serialize, O:for<'de> Deserialize<'de>
{
    /// Create a new empty graph
    /// This is done by giving a key to perform the hashes
    pub fn new() -> Self {
        Self{ 
            dag: BTreeMap::new(), 
            partial: false,
            top_layer: 0,
            heads: BTreeMap::new(), 
            topo: Vec::new(),
        }
    }

    //------------------------- HELPER FUNCTIONS -----------------------------

    /// Returns the length of the dag,
    /// i.e. the number of nodes in it
    pub fn len(&self) -> usize {
        return self.dag.len();
    }

    pub fn get_top_layer(&self) -> usize {
        return self.top_layer;
    }

    pub(crate) fn get_node(&self, id: &u64) -> Option<&Arc<Node<O>>> {
        self.dag.get(id)
    }

    /// get the head nodes of the dag
    pub fn get_heads(&self) -> Vec<u64> {
        return self.heads.keys().cloned().collect();
    }

    //------------------------- SPEC IMPLEMENTATION -----------------------------

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
    pub fn insert(&mut self, data:O, opt_cursor: Option<QueryCursor>)
        -> io::Result<(Arc<Node<O>>, QueryCursor)>
    {
        let mut cursor = match opt_cursor {
            None => QueryCursor::new(),
            Some(c) => c,
        };

        let parent_ids : Vec<u64> = cursor.heads.clone().into_iter().collect();
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

            cursor.heads.remove(parent_id);
            self.heads.remove(parent_id);
        }

        layer += 1;
        self.top_layer = cmp::max(self.top_layer, layer);

        let node = Node::new(data, parent_ids, Some(layer), Some(self.topo.len()));

        let nrf = Arc::new(node.clone());
        self.dag.insert(nrf.id.clone(), Arc::clone(&nrf));
        self.topo.push(Arc::clone(&nrf));
        self.heads.insert(nrf.id.clone(), Arc::clone(&nrf));

        cursor.heads.insert(nrf.id.clone());
        

        return Ok((nrf, cursor));
    }

    pub fn insert_node(&mut self, node:Node<O>, opt_cursor: Option<QueryCursor>)
        -> io::Result<(Arc<Node<O>>, QueryCursor)>
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
    
    /// Merge MerkleDag `dag` into `self`
    /// This means that all of the Nodes in `dag` which are not in `self`
    /// will be added to `self`
    ///
    /// Complefity $O(N+v)$ where 
    /// $N$ is the number of nodes and 
    /// $E$ is the number of conections between nodes
    /*#[deprecated]
    pub fn union(&mut self, dag:Self) {
        let mut stack : Vec<&Node<O>> = Vec::new();

        for (hash,node) in &dag.heads { 
            let x = self.dag.get(hash);
            if let Some(_) = x { 
                continue;
            }

            stack.push(&node);
            self.heads.insert(hash.clone(), node.clone());
        }


        loop { 
            if stack.is_empty() { break; } 
            let n = stack.pop().unwrap();
            info!("unionizing: {:?}", n.hash);

            for parent_hash in &n.parents { 
                let x = self.dag.get(parent_hash); 
                if let Some(_) = x { 
                    self.heads.remove(parent_hash);
                    continue;
                }

                let parent = dag.dag.get(parent_hash).unwrap(); 
                stack.push(&parent); 
            }

            self.dag.insert(n.hash.clone(), Arc::new(n.clone())); 
        }

        self.top_layer = cmp::max(self.top_layer, dag.top_layer);
    }*/

    /// Return a valid linearization of the Dag,
    /// i.e. a topological sort of the Dag
    ///
    /// Khan's topological sort algorithm was used to calculate the answer
    /// furthermore this function has complexity:
    /// - Time: $O(V*log(v) + E*log(v))$
    /// - Space: $O(V)$
    /// where $V$ is number of vertices of the graph and 
    /// $E$ the number of edges of the graph
    pub fn linearize(&self) -> Vec<Node<O>> {
        let mut res = vec![];
        let mut indegree = HashMap::new();
        let mut queue = VecDeque::new();

        for (_, node) in &self.dag {
            indegree.entry(node.id.clone()).or_insert(0);
            for parent_hash in &node.parents {
                *indegree.entry(parent_hash.clone()).or_insert(0) += 1;
            }
        }

        for (hash, node) in &self.dag {
            if let Some(0) = indegree.get(hash) {
                queue.push_back(node.clone());
            }
        }

        while let Some(node) = queue.pop_front() {
            res.push((*node).clone());

            for parent_hash in &node.parents {
                if let Some(parent_node) = self.dag.get(parent_hash) {
                    if let Some(indeg) = indegree.get_mut(&parent_node.id) {
                        *indeg -= 1;
                        if *indeg == 0 {
                            queue.push_back(parent_node.clone());
                        }
                    }
                } 
                else if !self.partial {
                    error!("Linearization error: parent {:?} of node {:?} is not in dag", parent_hash, node.id);
                }
            }
        }

        if res.len() != self.dag.len() {
            panic!("Cycle detected in the graph, linearization not possible");
        }

        return res;
    }

    
    /*
    fn subset_head<F>(&self,condition: F,mut dag:BTreeMap<Hash, Arc<Node<O>>>, head: Arc<Node<O>>) 
        -> BTreeMap<Hash, Arc<Node<O>>>
        where F: Fn(&Node<O>) -> bool
    {
        let mut queue:VecDeque<Arc<Node<O>>> = VecDeque::new();
        queue.push_back(Arc::clone(&head));
        dag.insert(head.hash.clone(), Arc::clone(&head));

        while let Some(node) = queue.pop_front() {
            for p_hash in node.parents {
                let parent = self.dag.get(&p_hash).unwrap();
                if !condition(parent) {
                    continue
                }

                let a = dag.insert(parent.hash.clone(), Arc::clone(parent));
                match a {
                    None => {
                        queue.push_back(parent.clone());
                    }
                    Some(_) => {}
                }
            }
        }


        return dag
    }

    /// Use a function to create a subset of the merkle dag,
    ///
    /// This means to select a subsection of the Dag based on 
    /// a Function `Fn`: `Node` -> `Bool`, where a certain head node is included in 
    /// the return dag if `Fn(node)` returns `true`
    ///
    /// A head node being included in the dag means that 
    /// every node that is a parent or parent of a parent of that head
    /// will be included in the return dag
    #[deprecated]
    pub fn subset<F>(&self, condition: F) -> Self
        where F: Fn(&Node<O>) -> bool
    {
        todo!("Should i reimplement this");
    }*/


    /// Use a function to create a subset of the merkle dag,
    ///
    /// This means to select a subsection of the Dag based on 
    /// a Function `Fn`: `Node` -> `Bool`, where a certain head node is included in 
    /// the return dag if `Fn(node)` returns `true`
    ///
    /// A head node being included in the dag means that 
    /// every node that is a parent or parent of a parent of that head
    /// will be included in the return dag
    ///
    /// The returning `QueryCursor` represents
    /// the list of heads that were queried, meaning that everything leading up 
    /// to these heads is already included in the graph
    pub fn query(&self, old_cursor: Option<QueryCursor>) -> (Vec<O>, QueryCursor)
    {
        let cursor = match old_cursor {
            Some(cursor) => cursor,
            None => QueryCursor::new()
        };
        let mut heap = BinaryHeap::<&Node<O>>::new();
        let size = self.topo.len() - cursor.index;
        let mut vis :Vec<bool> = vec![false; size.try_into().unwrap()];
        let mut res = Vec::<O>::new();

        for (_, node) in &self.heads {
            if node.index < cursor.index {
                continue
            }

            heap.push(node);
        }

        for head_hash in &cursor.heads{
            let head = self.get_node(&head_hash).unwrap();
            if head.index > cursor.index {
                vis[head.index - cursor.index] = true;
            }
        }

        while !heap.is_empty() {
            let top = heap.pop().expect("Heap should not be empty");

            for parent_hash in &top.parents {
                let parent = self.get_node(&parent_hash).unwrap();
                if parent.index < cursor.index {
                    continue
                }
                else if vis[top.index - cursor.index]{
                    vis[parent.index - cursor.index] = true;
                } 
                heap.push(parent);
            }
            
            if !vis[top.index - cursor.index]{
                res.push(top.data.clone());
            } 
        }

        let mut cursor = QueryCursor::new();
        cursor.set_heads(self.get_heads());
        cursor.index = self.topo.len();

        return (res.into_iter().rev().collect(), cursor)
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
        let dag = MerkleDag::<u8>::new();

        assert_eq!(dag.get_top_layer(), 0);
        assert_eq!(dag.len(), 0);
    }
    
    #[test]
    fn node_insertion(){
        let mut dag = MerkleDag::<u8>::new();
        let node1 = Node::new(1, vec![], None, None);

        assert!(dag.insert_node(node1, None).is_ok(), 
            "Node was not added correctly");
        assert_eq!(dag.len(), 1);

        let node2 = Node::new(2, vec![3], None, None);
        assert!(dag.insert_node(node2, None).is_err(), 
            "Node whose parents were not in the dag was added");
        assert_eq!(dag.len(), 1);
    }

    #[test]
    fn querying_loose_heads(){
        let mut dag = MerkleDag::<u8>::new();

        let mut c = QueryCursor::new();

        for i in 0..10 as usize {
            let node = Node::new(i as u8, vec![], None, None);
            c = dag.insert_node(node, Some(c)).unwrap();
            assert_eq!(c.heads.len(), i+1, 
                "the cursor should include all heads of the dag");
        }

        let (changes, cursor) = dag.query(None);
        let len1 = changes.len();
        assert_eq!(len1, dag.len());
            
        for i in 10..20 as usize {
            let node = Node::new(i as u8, vec![], None, None);
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
            let node = Node::new(i as u8,vec![node.id.clone()], None, None);
            let nid = node.id.clone();
            loose = dag.insert_node(node, Some(loose)).unwrap();

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
