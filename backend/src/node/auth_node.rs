use std::fmt::{Formatter, Debug, self};
use std::vec::Vec;
use std::hash::{Hash, Hasher};
use serde::{
    self,
    Serialize, Deserialize,
};
use serde_with::{serde_as};
use hmac::Hmac;
use digest::Mac;
use sha3::Sha3_256;

use crate::traits::Node;

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct AuthNode<O> 
    where O:Clone, O:Debug, O:Hash,
{
    pub id: Vec<u8>,
    pub parents: Vec<Vec<u8>>,
    pub data: O,
}

impl<O> Node<O> for AuthNode<O>
    where O:Clone, O:Debug, O:Hash, O:Serialize
{
    type Key = Vec<u8>;
    type Hash = Vec<u8>;

    fn new(key: &Vec<u8>, data: O, parents: Vec<Vec<u8>>) -> Self{
        let mut mac = Hmac::<Sha3_256>::new_from_slice(key)
                .expect("HMAC can take key of any size");

        for parent_id in &parents {
            mac.update(parent_id);
        }

        let bytes = serde_json::to_vec(&data).expect("serialization failed"); 
        mac.update(&bytes);

        let hash = mac.finalize().into_bytes().to_vec();

        Self {
            id:hash,
            data,
            parents,
        }
    }

    fn verify(&self, key: &Vec<u8>) -> bool {
        let mut mac = Hmac::<Sha3_256>::new_from_slice(key)
                .expect("HMAC can take key of any size");

        for parent_id in &self.parents {
            mac.update(parent_id);
        }

        let bytes = serde_json::to_vec(&self.data).expect("serialization failed"); 
        mac.update(&bytes);

        let hash = mac.finalize().into_bytes().to_vec();
        return self.id == hash;
    }

    fn get_id(&self) -> Self::Hash {self.id.clone()}
    fn get_data(&self) -> O {self.data.clone()}
    fn get_parents(&self) -> Vec<Self::Hash> {self.parents.clone()}
}

impl<O> Hash for AuthNode<O> 
    where O:Clone, O:Debug, O:Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.id);
    }
}

impl<O> Debug for AuthNode<O>
    where O:Clone, O:Debug, O:Hash,
{
   fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("parents",&self.parents)
            .field("data",  &self.data)
            .finish()
    }
}
