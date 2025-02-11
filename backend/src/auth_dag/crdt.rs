use automerge::{
    AutoCommit, transaction::Transactable, 
    ReadDoc, ObjType, ObjId,
    Change,

};
use serde_json;
use crate::auth_dag::AuthDag;


pub struct CRDT {
    doc: AutoCommit,
    dag: AuthDag,
    cmap: ObjId,
}

/*fn hash_to_key(bytes: &Hash) -> String {
 *   String::from_utf8_lossy(&bytes).to_string()
 *}
 */

impl CRDT {
    pub async fn new(username: &str, password: &str) -> Self {
        let mut doc = AutoCommit::new();
        let cmap = doc.put_object(automerge::ROOT, "map", ObjType::Map).expect("failed to put map in autocommit");

        CRDT {
            doc,
            dag: AuthDag::new(username, password).await,
            cmap,
        }
    }

    pub fn update(&mut self, key: &str, value: &str){
        self.doc.put(&self.cmap, key, value).expect("failed to insert in map");
    }

    pub async fn save(&mut self){
        let ochange = self.doc.get_last_local_change();
        match ochange {
            None => {},
            Some(change) => {
                let s = serde_json::to_string(&change.raw_bytes()).expect("failed to serialize change");
                self.dag.send_update(s).await;
            }
        }
    }



    pub fn query(&mut self){
        let ser_change = self.dag.query();
        let changes: Vec<Change> = ser_change.into_iter().map(|data: String| {
            let deser : &[u8] = serde_json::from_str(&data).expect("failed to deserialize");
            let change = Change::from_bytes(deser.to_vec()).expect("failed to transform into change");
            return change;
        }).collect();
        
        let _result = self.doc.apply_changes(changes);
        todo!("make sure result is taken care of ");
    }

    pub fn pretty_print_dag(&self) -> String {
        self.dag.pretty_print_dag()
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
