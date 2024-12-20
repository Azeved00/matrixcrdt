use std::sync::Arc;
use sha3::Sha3_256;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use matrix_sdk::{
    config::SyncSettings,
    event_handler::Ctx,
    sync::SyncResponse,
    ruma::events::{
        macros::EventContent,
        //room::{
            //member::StrippedRoomMemberEvent,
            //message::{
            //    SyncRoomMessageEvent, 
            //    OriginalSyncRoomMessageEvent
            //},
        //},
    },
    ruma::RoomId,
    Room,  RoomState,
    Client
};

use crate::auth_dag::merkle_dag::{auth::AuthMerkleDag, node::Node, dag::QueryRecord};

type DagReference = Arc<RwLock<AuthMerkleDag<Sha3_256, String>>>; 


#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "fcup.acrdt.update", kind = MessageLike)]
struct UpdateEventContent {
    cmd: Node<String>,
    author: String,
    version: u8,
}

pub struct AuthDag
{
    room: Room,
    client: Arc<RwLock<Client>>,
    user: String,
    dag: DagReference,
    sync_settings: SyncResponse,

}

impl AuthDag
{
    const ROOM_ID : &str = "!GXNPdYSjbFRDdXdyRK:matrix.org";
    const HOMESERVER : &str = "https://matrix.org";
    const VERSION: u8 = 3;

    ///create a new Crdt
    pub async fn new(username: &str, password: &str) -> Self {
        println!("logging in");
        let client = Client::builder()
            .homeserver_url(Self::HOMESERVER.to_string())
            .build()
            .await.expect("failed to connect to homeserver");

        client
            .matrix_auth()
            .login_username(username, password)
            .initial_device_display_name("getting started bot")
            .await.expect("authenticated failed");

        println!("logged in as {username}");


        let dag = AuthMerkleDag::new(password.into());
        let context: DagReference = Arc::new(RwLock::new(dag));
        client.add_event_handler_context(Arc::clone(&context));
        client.add_event_handler(self::map_on_update);

        let response = client.sync_once(Default::default()).await.unwrap();

        let oroom_id = RoomId::parse(Self::ROOM_ID).expect("failed to parse room id");
        let room = client.get_room(&oroom_id).expect("Room not found"); 

        Self {
            room,
            client: Arc::new(RwLock::new(client)),
            user: username.to_string(),
            dag: context,
            sync_settings: response,
        }
    }

    pub async fn send_update(&self, cmd: String) {
        let dag= self.dag.read().await;
        let node = dag.gen_node(cmd);

        let content = UpdateEventContent {
            cmd: node,
            author: self.user.clone(),
            version: Self::VERSION,
        };

        let result = self.room.send(content).await;
        if result.is_err() {
            panic!("An error occurred: {:?}", result.unwrap_err());
        }
    }

    pub async fn start_sync(&self) {
        println!("Start Syncing");
        let settings = SyncSettings::default().token(self.sync_settings.next_batch.clone());
        let client = self.client.write().await;
        let _ = client.sync(settings.clone()).await;
    }

    pub fn query<F>(&self, func: F, log: Option<QueryRecord>) -> QueryRecord
        where F: Fn(&Node<String>)
    {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            runtime.block_on(async {
                let map = self.dag.read().await;
                map.query(func, log)
            })
        })
    }

    /// Pretty print function for a HashMap
    pub fn pretty_print_dag(&self) -> String
    {
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            runtime.block_on(async {
                let map = self.dag.read().await;
                let mut output = String::new();
                output.push_str("{\n");
                for value in map.linearize() {
                    output.push_str(&format!("  {:#?},\n", value));
                }
                output.push('}');
                output
            })
        })
    }
}

async fn map_on_update(event: SyncUpdateEvent, room: Room, ctx: Ctx<DagReference>) {
    //println!("received update");
    if room.state() != RoomState::Joined {
        return;
    }
    
    let original = event.as_original()
        .expect("Cant get the original of received event");
    if original.content.version != AuthDag::VERSION {
        return
    }


    let mut dag = ctx.write().await;
    dag.add_node(original.content.cmd.clone());
}
