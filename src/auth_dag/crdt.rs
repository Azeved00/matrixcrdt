use automerge::{ObjType, AutoCommit, transaction::Transactable, ReadDoc};
use crate::auth_dag::{AuthDag,Hash,merkle_dag::dag::QueryRecord};


pub struct CRDT {
    doc: AutoCommit,
    dag: AuthDag,
    log: QueryRecord,
}

fn hash_to_key(bytes: &Hash) -> String {
    String::from_utf8_lossy(&bytes).to_string()
}

impl CRDT {
    pub async fn new(username: &str, password: &str) -> Self {
        CRDT {
            doc: AutoCommit::new(),
            dag: AuthDag::new(username, password).await,
            log: QueryRecord::default(),
        }
    }

    pub async fn update(&self, value: String) {
        self.dag.send_update(value).await;
    }

    pub fn query(&mut self){
        let log = self.dag.query(|node| {
            self.doc.put(
                automerge::ROOT, 
                hash_to_key(&node.hash),
            node.data.clone()).unwrap();
        }, Some(self.log.clone()));
        self.log = log;
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
