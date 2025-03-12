use log::{info,error};
use std::fmt::{Formatter, Debug, Result};
use std::vec::Vec;
use std::collections::BTreeMap;
use std::collections::{HashSet, HashMap, BinaryHeap, VecDeque};
use std::cmp;
use std::io;
use serde::{Serialize, Deserialize};

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
#[derive(Serialize, Deserialize,Clone)]
pub struct MerkleDag<O>
  where O: Clone, O: Into<Vec<u8>>, O: Debug,
{
    pub(super) dag: BTreeMap<Hash, Node<O>>,
    partial: bool,
    top_layer: usize,
    pub(super) heads: BTreeMap<Hash, Node<O>>,
}


impl<O> MerkleDag<O> 
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
    pub fn get_node(&self, hash: &Hash) -> Option<&Node<O>> {
        self.dag.get(hash)
    }

    /// Add a node to the network
    ///
    /// return the hash of the newly added hash
    /// This method has temporal complexity: O(p. log n) where
    ///     p is number of parents of the node
    ///     n is the number of nodes of the graph
    pub fn add_node(&mut self, node: Node<O>, opt_cursor: Option<QueryCursor>) -> io::Result<QueryCursor>
    {
        if self.dag.contains_key(&node.hash){
            return match opt_cursor {
                Some(cursor) => Ok(cursor),
                None => Ok(QueryCursor {set: HashSet::from([node.hash.clone()]) }),
            }
        }

        for parent_hash in &node.parents {
            let parent = self.dag.get(parent_hash);
            match parent {
                None => {
                    return Err(io::Error::new(io::ErrorKind::Other, "Not all parents of the node are in the Dag"));
                }
                Some(_) => {}
            }


            self.heads.remove(parent_hash);
        }

        self.top_layer = cmp::max(self.top_layer, node.layer);
        self.dag.insert(node.hash.clone(),node.clone());
        self.heads.insert(node.hash.clone(),node.clone());

        return match opt_cursor {
            None => Ok(QueryCursor {set: HashSet::from([node.hash.clone()]) }),
            Some(c) => {
                let mut cursor = c.clone();
                for parent in &node.parents {
                    cursor.set.remove(parent);
                }
                cursor.set.insert(node.hash.clone());

                return Ok(cursor);
            }
        };
    }
    
    /// Merge MerkleDag `dag` into `self`
    /// This means that all of the Nodes in `dag` which are not in `self`
    /// will be added to `self`
    ///
    /// Complefity $O(N+v)$ where 
    /// $N$ is the number of nodes and 
    /// $E$ is the number of conections between nodes
    #[deprecated]
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

            self.dag.insert(n.hash.clone(), n.clone()); 
        }

        self.top_layer = cmp::max(self.top_layer, dag.top_layer);
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
            res.push(node.clone());

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
    
    fn subset_head<F>(&self,condition: F,mut dag:BTreeMap<Hash, Node<O>>, head: Node<O>) -> BTreeMap<Hash, Node<O>>
        where F: Fn(&Node<O>) -> bool
    {
        let mut queue:VecDeque<Node<O>> = VecDeque::new();
        queue.push_back(head.clone());
        dag.insert(head.hash.clone(), head);

        while let Some(node) = queue.pop_front() {
            for p_hash in node.parents {
                let parent = self.dag.get(&p_hash).unwrap();
                if !condition(parent) {
                    continue
                }

                let a = dag.insert(parent.hash.clone(), parent.clone());
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
        let mut new_dag = BTreeMap::new();
        let mut new_heads = BTreeMap::new();


        for (hash, node) in &self.heads {
            if condition(node) {
                info!("head included in subset");
                new_heads.insert(hash.clone(), node.clone());
                new_dag = self.subset_head(&condition, new_dag, node.clone());
            }
        }

        Self {
            dag: new_dag,
            heads: new_heads,
            top_layer: 0,
            partial: true,
        }
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
    ///
    /// The returning `QueryCursor` represents
    /// the list of heads that were queried, meaning that everything leading up 
    /// to these heads is already included in the graph
    pub fn query(&self, old_cursor: Option<QueryCursor>) -> (Vec<O>, QueryCursor)
    {
        let cursor = match old_cursor {
            Some(cursor) => cursor,
            None => {
                QueryCursor {set: HashSet::new() }
            }
        };
        let mut heap = BinaryHeap::<&Node<O>>::new();
        let mut res = Vec::<O>::new();

        for (_, node) in &self.heads {
            if cursor.contains(&node.hash) { 
                continue; 
            }
            heap.push(node);
        }
        
        while !heap.is_empty() {
            let top = heap.pop().expect("Heap should be empty");
            for parent_hash in &top.parents {
                if cursor.contains(&parent_hash) {
                    continue;
                }
                let parent = self.get_node(&parent_hash).unwrap();
                heap.push(parent);
            }
            
            res.push(top.data.clone());
        }
        

        let heads : Vec<Vec<u8>> = self.heads.keys().cloned().collect();
        let cursor = QueryCursor {set: heads.into_iter().collect() };
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

        for i in 0..10 as usize {
            let node = Node::<Vec<u8>>::new::<Sha3_256>(&key, & i.to_be_bytes().into(), &vec![], 0);
            dag.add_node(node, None).unwrap();
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

        let (changes, cursor2) = dag.query(Some(cursor));
        assert_eq!(changes.len() + len1, dag.len(), "Querying with cursor outputs the new nodes");

        let (changes, _c) = dag.query(Some(cursor2));
        assert_eq!(changes.len() , 0 , "Querying twice in a row (with cursor) outputs empty changes array");
    }
}
