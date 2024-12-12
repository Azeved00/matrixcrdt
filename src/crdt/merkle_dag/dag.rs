use log::{info,error};
use std::fmt::{Formatter, Debug, Result};
use std::vec::Vec;
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet, VecDeque};
use digest::{
    Digest, HashMarker,
    core_api::*,
    typenum::*,
    block_buffer::Eager,
    consts::U256,
};
use serde::{Serialize, Deserialize};

use super::node::Node;
use super::Hash;


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
    dag: BTreeMap<Hash, Node<O>>,
    partial: bool,
    pub (super)heads: BTreeMap<Hash, Node<O>>,
}

#[derive(Clone, Default)]
pub struct QueryRecord
{
    set: HashSet<Hash>
}

impl<O> MerkleDag<O> 
    where O: Clone, O: Into<Vec<u8>>, O: Debug,
{
    /// Create a new empty graph
    /// This is done by giving a key to perform the hashes
    pub fn new() -> Self {
        Self{ 
            dag: BTreeMap::new(), 
            partial: true,
            heads: BTreeMap::new(), 
        }
    }

    /// Returns the length of a node,
    /// i.e. the number of nodes in it
    pub fn len(&self) -> usize {
        return self.dag.len();
    }

    /// Add a node to the network
    ///
    /// return the hash of the newly added hash
    /// This method has temporal complexity: O(p. log n) where
    ///     p is number of parents of the node
    ///     n is the number of nodes of the graph
    pub fn add_node(&mut self, node: Node<O>){
        for parent_hash in &node.parents {
            let parent = self.dag.get(parent_hash);
            match parent {
                None => {
                    if !self.partial {
                        error!("NOT ALL PARENTS ARE IN THE NODE");
                    }
                    return;
                }
                Some(_) => {}
            }


            self.heads.remove(parent_hash);
        }

        self.dag.insert(node.hash.clone(),node.clone());
        self.heads.insert(node.hash.clone(),node.clone());
    }

    /// This function verifies the input `dag` against this dag
    ///
    /// This function assumes `self` is fully verified and 
    /// seeks to verify the nodes of `dag` that are not in `self`
    ///
    /// for a graph to be verified:(1) the hashes in each f the nodes need to be correct and
    /// (2) for all nodes, all parents are inside the graph (i.e. the graph is total)
    pub fn verify_other<D>(&self, dag:Self, key: &Vec<u8>) -> bool
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
        false
    }

    /// This function verifies the DAG, this means that
    /// 1. the hashes in each of the nodes are correct and
    /// 2. all parents of all nodes are inside the graph
    /// if these conditions are met then `true` is returned
    pub fn verify<D>(&self, key: &Vec<u8>) -> bool
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
        self.dag.values().all(|node| 
            node.verify::<D>(key.clone()) &&
            node.parents.iter().all(|parent_hash| {
                if let Some(_) = self.dag.get(parent_hash) {
                    true
                } else {
                    false
                }
        }))
    }
    
    /// Merge MerkleDag `dag` into `self`
    /// This means that all of the Nodes in `dag` which are not in `self`
    /// will be added to `self`
    ///
    /// Complefity $O(N+v)$ where 
    /// $N$ is the number of nodes and 
    /// $E$ is the number of conections between nodes
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
            partial: true,
        }
    }

    /// Get a Node with a cetain hash from the dag
    pub fn get_node(&self, hash: &Hash) -> Option<&Node<O>> {
        self.dag.get(hash)
    }

    fn query_head<F>(&self,func: &F, record:&QueryRecord, head: &Node<O>)
        where F: Fn(&Node<O>)     
    {
        let mut queue:VecDeque<&Node<O>> = VecDeque::new();
        queue.push_back(head);

        while let Some(node) = queue.pop_front() {
            func(&node);

            for p_hash in &node.parents {
                if (&record.set).contains(p_hash) {
                    continue
                }

                let parent = self.dag.get(p_hash).unwrap();
                queue.push_back(&parent);
            }
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
    pub fn query<F>(&self, func: F,orecord: Option<QueryRecord>) -> QueryRecord
        where F: Fn(&Node<O>)
    {
        let record = match orecord {
            None => QueryRecord { set: HashSet::new() },
            Some(rec) => rec,
        };

        for (hash, node) in &self.heads {
            if (&record.set).contains(hash) {
                continue;
            }

            self.query_head(&func, &record, &node);
        }

        let new_set :HashSet<Hash> = self.heads.keys().cloned().collect();

        QueryRecord{ set: new_set }
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
