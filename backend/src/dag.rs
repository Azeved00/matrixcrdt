use log::error;
use std::fmt::{Formatter, Debug, Result};
use std::vec::Vec;
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet, BinaryHeap, VecDeque};
use std::cmp;
use std::io;
use std::sync::Arc;

use super::node::Node;
use super::Hash;
use super::QueryCursor;


/// The Merkle dag structure,
///
/// The structure makes use of 2 binary tree map structures.
///
/// Furthermore, the structure differentiates between a partial and a total Dag
/// a total dag is a dag such that for all nodes all parents are in the dag while
/// a partial dag is a dag in which the parents of some nodes are not inside the dag
#[derive(Clone)]
pub struct MerkleDag<O>
  where O: Clone, O: Into<Vec<u8>>, O: Debug,
{
    pub(super) dag: BTreeMap<Hash, Arc<Node<O>>>,
    pub(super) heads: BTreeMap<Hash, Arc<Node<O>>>,
    pub(super) topo: Vec<Arc<Node<O>>>,
    partial: bool,
    top_layer: usize,
}


impl<O> MerkleDag< O> 
    where O: Clone, O: Into<Vec<u8>>, O: Debug,
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

    /// Returns the length of the dag,
    /// i.e. the number of nodes in it
    pub fn len(&self) -> usize {
        return self.dag.len();
    }

    pub fn get_top_layer(&self) -> usize {
        return self.top_layer;
    }

    /// Get a Node with a cetain hash from the dag
    pub fn get_node(&self, hash: &Hash) -> Option<&Arc<Node<O>>> {
        self.dag.get(hash)
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
    pub fn add_node(&mut self, node: Node<O>, opt_cursor: Option<QueryCursor>)
        -> io::Result<QueryCursor>
    {
        if self.dag.contains_key(&node.hash){
            return match opt_cursor {
                Some(cursor) => Ok(cursor),
                None => {
                    let mut c = QueryCursor::new();
                    c.heads.insert(node.hash.clone());
                    Ok(c)
                },
            }
        }
        let mut cursor = match opt_cursor {
            None => QueryCursor::new(),
            Some(c) => c,
        };

        for parent_hash in &node.parents {
            let parent = self.dag.get(parent_hash);
            match parent {
                None => {
                    return Err(io::Error::new(io::ErrorKind::Other, "Not all parents of the node are in the Dag"));
                }
                Some(_) => {}
            }

            cursor.heads.remove(parent_hash);
            self.heads.remove(parent_hash);
        }

        self.top_layer = cmp::max(self.top_layer, node.layer);

        let mut nn = node.clone();
        nn.index = self.topo.len();
        let nrf = Arc::new(nn);
        self.dag.insert(node.hash.clone(), Arc::clone(&nrf));
        self.topo.push(Arc::clone(&nrf));
        self.heads.insert(node.hash.clone(), Arc::clone(&nrf));

        cursor.heads.insert(node.hash.clone());

        return Ok(cursor);
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
            indegree.entry(node.hash.clone()).or_insert(0);
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
                    if let Some(indeg) = indegree.get_mut(&parent_node.hash) {
                        *indeg -= 1;
                        if *indeg == 0 {
                            queue.push_back(parent_node.clone());
                        }
                    }
                } 
                else if !self.partial {
                    error!("Linearization error: parent {:?} of node {:?} is not in dag", parent_hash, node.hash);
                }
            }
        }

        if res.len() != self.dag.len() {
            panic!("Cycle detected in the graph, linearization not possible");
        }

        return res;
    }

    /// get the head nodes of the dag
    pub fn get_heads(&self) -> Vec<Hash> {
        return self.heads.keys().cloned().collect();
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
        let mut vis :Vec<bool> = vec![false; self.topo.len() - cursor.index];
        let mut res = Vec::<O>::new();

        for (_, node) in &self.heads {
            if node.index < cursor.index {
                continue
            }

            heap.push(node);
        }

        for head_hash in &cursor.heads{
            let head = self.get_node(&head_hash).unwrap();
            if head.index>cursor.index {
                vis[head.index-cursor.index] = true;
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

        let heads : HashSet<Vec<u8>> = self.heads.keys().cloned().collect();
        let mut cursor = QueryCursor::new();
        cursor.heads = heads;
        cursor.index = self.topo.len();

        return (res.into_iter().rev().collect(), cursor)
    }
}


impl<O>Debug for MerkleDag<O> 
    where O: Clone, O: Into<Vec<u8>>, O:Debug,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let linearization = self.linearize();
        f.debug_struct("Merkle Dag")
            .field("length", &self.dag.len())
            .field("partial", &self.partial)
            .field("heads",  &self.heads)
            .field("graph",  &linearization)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha3::Sha3_256;

    #[test]
    fn initialization() {
        let dag = MerkleDag::<Vec<u8>>::new();

        assert_eq!(dag.get_top_layer(), 0);
        assert_eq!(dag.len(), 0);
    }
    
    #[test]
    fn insertion(){
        let mut dag = MerkleDag::<Vec<u8>>::new();

        let key : Vec<u8> = "".to_string().into();
        let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, &vec![], &vec![], 0);

        assert!(dag.add_node(node, None).is_ok(), "Node was not added correctly");
        assert_eq!(dag.len(), 1);

        let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, &vec![1,2,3], &vec![vec![1]], 0);
        assert!(dag.add_node(node, None).is_err(), "Node whose parents were not in the dag was added");
        assert_eq!(dag.len(), 1);
    }

    #[test]
    fn querying(){
        let mut dag = MerkleDag::<Vec<u8>>::new();
        let key : Vec<u8> = "".to_string().into();

        let mut t = QueryCursor::new();

        for i in 0..10 as usize {
            let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, & i.to_be_bytes().into(), &vec![], 0);
            t = dag.add_node(node, Some(t)).unwrap();
            assert_eq!(t.heads.len(), i+1, "the cursor should include the heads of all nodes");
        }

        let (changes, cursor) = dag.query(None);
        let len1 = changes.len();
        assert_eq!(len1, dag.len());
            
        for i in 10..20 as usize {
            let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, & i.to_be_bytes().into(), &vec![], 0);
            dag.add_node(node, None).unwrap();
        }

        let (changes, _c) = dag.query(None);
        assert_eq!(changes.len(), dag.len(), "Querying without cursor, outputs the full dag");

        println!("{:}", cursor.index);
        let (changes, cursor2) = dag.query(Some(cursor));
        println!("{:}", cursor2.index);
        assert_eq!(changes.len() + len1, dag.len(), "Querying with cursor outputs the new nodes");

        let (changes, _c) = dag.query(Some(cursor2));
        assert_eq!(changes.len() , 0 , "Querying twice in a row (with cursor) outputs empty changes array");
    }
    
    #[test]
    // when querying dag with multiple branches only new nodes should be queried 
    fn query_multiple_branches() {
        let mut dag = MerkleDag::<Vec<u8>>::new();
        let key : Vec<u8> = "".to_string().into();
        let mut cursor = QueryCursor::new();

        let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, & (0_usize).to_be_bytes().into(), &vec![], 0);
        let mut last_hash: Vec<u8> = node.hash.clone();
        cursor = dag.add_node(node, Some(cursor)).unwrap();

        for i in 0..5 as usize {
            let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, & i.to_be_bytes().into(), &vec![last_hash], 0);
            last_hash = node.hash.clone();
            cursor = dag.add_node(node, Some(cursor)).unwrap();
        }

        let branch1 = Node::<Vec<u8>>::new::<Sha3_256>(&key, & (7_usize).to_be_bytes().into(), &vec![last_hash.clone()], 0);
        let cursor1 = dag.add_node(branch1, Some(cursor.clone())).unwrap();
        println!("dag heads {:?}", dag.get_heads());
        println!("cursor {:?}", cursor1);

        let branch2 = Node::<Vec<u8>>::new::<Sha3_256>(&key, & (8_usize).to_be_bytes().into(), &vec![last_hash.clone()], 0);
        let cursor2 = dag.add_node(branch2, None).unwrap();
        println!("");
        println!("dag heads {:?}", dag.get_heads());
        println!("cursor {:?}", cursor2);

        let (res1, _)= dag.query(Some(cursor1));
        println!("");
        println!("result {:?}", res1);
        assert!(res1.len() == 1, "The result of the query should be an array with only 1 element(the new node)");

        let (res2, _)= dag.query(Some(cursor2));
        println!("");
        println!("result {:?}", res2);
        assert!(res2.len() == 1, "The result of the query should be an array with only 1 element(the new node)");

    }

    #[test]
    // when creating branches the heads of the branches should stay in the cursor
    fn query_cursor_branching() {
        let mut dag = MerkleDag::<Vec<u8>>::new();
        let key : Vec<u8> = "".to_string().into();

        let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, & (400_usize).to_be_bytes().into(), &vec![], 0);
        let mut last_hashes: Vec<Vec<u8>> = vec![node.hash.clone()];
        let mut cursor = dag.add_node(node, None).unwrap();
        assert!(cursor.contains(&last_hashes[0]), 
            "the first cursor should have the first node added");

        for i in 0..10 as usize {
            let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, & i.to_be_bytes().into(), &vec![], 0);
            let nhash = node.hash.clone();
            cursor = dag.add_node(node, Some(cursor)).unwrap();

            println!("heads {:?}", dag.get_heads());
            println!("last {:?}", last_hashes);
            println!("cursor {:?}", cursor);

            for hash in &last_hashes {
                assert!(cursor.contains(hash),
                    "the new cursor should include the old nodes");
            }
            assert!(cursor.contains(&nhash), 
                "the new cursor should include the new node");

            last_hashes.push(nhash);
        }
    }

    #[test]
    // when adding to existing branches only the head should be there
    fn query_cursor_single_branch() {
        let mut dag = MerkleDag::<Vec<u8>>::new();
        let key : Vec<u8> = "".to_string().into();

        let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, & (0_usize).to_be_bytes().into(), &vec![], 0);
        let mut last_hashes: Vec<Vec<u8>> = vec![node.hash.clone()];
        let mut last_hash: Vec<u8> = node.hash.clone();
        let mut cursor = dag.add_node(node, None).unwrap();
        assert!(cursor.heads.get(&last_hashes[0]).is_some(), "the first cursor should have the first node added");

        for i in 0..10 as usize {
            let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, & i.to_be_bytes().into(), &vec![last_hash], 0);
            let hash = node.hash.clone();
            cursor = dag.add_node(node, Some(cursor)).unwrap();

            println!("heads {:?}", dag.get_heads());
            println!("last {:?}", last_hashes);
            println!("cursor {:?}", cursor);

            for hash in &last_hashes {
                assert!(!cursor.contains(hash), "the new cursor should NOT include the old nodes");
            }
            assert!(cursor.contains(&hash), "the new cursor should include the new node");

            last_hashes.push(hash.clone());
            last_hash = hash;
        }
    }
}
