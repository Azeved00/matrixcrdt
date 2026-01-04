use std::collections::HashSet;
use std::{
    self,
    fmt, cmp, hash
};

#[derive(Clone, Default)]
pub struct QueryCursor<H = Vec<u8>> 
    where H: cmp::Eq + hash::Hash
{
    pub (crate) heads: HashSet<H>,
    pub (crate) index: usize,
}

impl<H> QueryCursor<H> 
    where H: cmp::Eq + hash::Hash
{
    pub fn new() -> Self {
        QueryCursor {
            heads: HashSet::new(),
            index: 0,
        }
    }

    /// Weather a hash is referenced in this cursor
    pub fn contains(&self, id: &H) -> bool {
        self.heads.contains(id)
    }
    
    pub fn set_heads(&mut self, heads: Vec<H>) {
        self.heads = HashSet::from_iter(heads);
    }
}

impl<H> fmt::Debug for QueryCursor<H>
where
    H: fmt::Debug + cmp::Eq + cmp::Ord + hash::Hash
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut heads: Vec<&H> = self.heads.iter().collect();
        heads.sort();

        let mut ds = f.debug_struct("QueryCursor");
        ds.field("index", &self.index);
        ds.field("heads", &heads);
        ds.finish()
    }
}
