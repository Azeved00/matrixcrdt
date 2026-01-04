use std::io::Result;
use std::sync::Arc;

use crate::QueryCursor;


pub trait Node<O>
{
    type Key;
    type Hash;

    fn new(key: &Self::Key, data: O, parents: Vec<Self::Hash>) -> Self;
    fn verify(&self, key: &Self::Key) -> bool;

    fn get_id(&self) -> Self::Hash;
    fn get_data(&self) -> O;
    fn get_parents(&self) -> Vec<Self::Hash>;
}

pub trait Client<O> 
{
    type Key;
    type NodeImpl: Node<O, Key = Self::Key>;

    fn new(key: Self::Key, client_id: u64) -> Self;

    fn insert(&mut self, data:O) -> Result<Arc<Self::NodeImpl>>;

    fn query(&self) -> Vec<O>;

    //fn merge(&mut self,d : Vec<Self::NodeImpl>) -> Result<()>;
    fn merge(&mut self,d : Self) -> Result<()>;
}


pub trait Replica<N> 
{
    fn new(crypto: Vec<u8>, client_id: u64) -> Self;

    fn insert(&mut self, update_data:N) -> Result<()>;

    fn merge(&mut self,d : Vec<N>) -> Result<QueryCursor>;

    fn query(&self, cursor: QueryCursor) -> (Vec<N>, QueryCursor);
}

