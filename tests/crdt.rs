use automerge::{
    AutoCommit, transaction::Transactable, 
    ReadDoc, ObjType, ObjId,
    Change,

};
use serde_json;
use sha3::Sha3_256;

use matrix_acrdt::auth_dag::merkle_dag::{
    auth::AuthMerkleDag, node::Node, QueryCursor
};

type Dag = AuthMerkleDag<Sha3_256, String>; 

pub struct CRDT {
    doc: AutoCommit,
    dag: Dag,
    cmap: ObjId,
    cursor: QueryCursor,

}

/*fn hash_to_key(bytes: &Hash) -> String {
 *   String::from_utf8_lossy(&bytes).to_string()
 *}
 */

impl CRDT {
    pub async fn new(_username: &str, password: &str) -> Self {
        let mut doc = AutoCommit::new();
        let cmap = doc.put_object(automerge::ROOT, "map", ObjType::Map).expect("failed to put map in autocommit");

        CRDT {
            doc,
            dag: AuthMerkleDag::new(password.into()),
            cmap,
            cursor: QueryCursor::default(),
        }
    }

    pub fn update(&mut self, key: &str, value: &str){
        self.doc.put(&self.cmap, key, value).expect("failed to insert in map");
    }

    pub async fn save(&mut self) -> Option<Node<String>> {
        let ochange = self.doc.get_last_local_change();
        match ochange {
            None => None,
            Some(change) => {
                let s = serde_json::to_string(&change.raw_bytes()).expect("failed to serialize change");
                let (node,cursor) = self.dag.gen_node(s, Some(self.cursor.clone()));
                self.cursor = cursor;
                return Some(node);
            }
        }
    }



    pub fn query(&mut self){
        let ser_change = self.dag.query(Some(self.cursor));
        let changes: Vec<Change> = ser_change.into_iter().map(|data: String| {
            let deser : &[u8] = serde_json::from_str(&data).expect("failed to deserialize");
            let change = Change::from_bytes(deser.to_vec()).expect("failed to transform into change");
            return change;
        }).collect();
        
        let _result = self.doc.apply_changes(changes);
        todo!("make sure result is taken care of ");
    }

    pub fn pretty_print_dag(&self) -> String {
        let mut output = String::new();
        output.push_str("{\n");
        for value in self.dag.linearize() {
            output.push_str(&format!("  {:#?},\n", value));
        }
        output.push('}');
        output
    }

    pub fn pretty_print_doc(&self) -> String {
        let mut result = String::new();

        for key in self.doc.keys(automerge::ROOT) {
            if let Some((value, _)) = self.doc.get(automerge::ROOT, &key).unwrap() {
                result.push_str(&format!("{:?}\n", value));
            }
        }

        result
    }
}
