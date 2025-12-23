use std::io::Result;

use crate::QueryCursor;


pub trait Node<O>
{
    type Key;
    type Hash;

    fn new(key:Key, data: O, parents: Vec<Hash>) -> Self;
}

pub trait Client<O> 
{
    type Key;
    type NodeImpl: Node<O, Key = Self::Key>;

    fn new(key: Key, client_id: u64) -> Self;

    fn insert(&mut self, data:O) -> Result<(NodeImpl)>;

    fn query(&self) -> Vec<O>;


    fn merge(&mut self,d : Vec<NodeImpl>) -> Result<()>;
}


pub trait Replica<N> 
{
    fn new(crypto: Vec<u8>, client_id: u64) -> Self;

    fn insert(&mut self, update_data:N) -> Result<()>;

    fn merge(&mut self,d : Vec<N>) -> Result<QueryCursor>;

    fn query(&self, cursor: QueryCursor) -> (Vec<N>, QueryCursor);
}

