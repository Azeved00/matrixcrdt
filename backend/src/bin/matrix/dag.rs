use std::sync::Arc;
use sha3::Sha3_256;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use matrix_sdk::{
    config::SyncSettings,
    event_handler::Ctx,
    ruma::events::macros::EventContent,
    ruma::RoomId,
    Room,  RoomState,
    Client
};

use auth_crdt::{auth::AuthMerkleDag, node::Node, QueryCursor};

type DagReference = Arc<RwLock<AuthMerkleDag<Sha3_256, String>>>; 


#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "fcup.acrdt.update", kind = MessageLike)]
struct UpdateEventContent
{
    cmd: Node<String>,
    author: String,
    version: u8,
}

pub struct AuthDag
{
    room: Room,
    user: String,
    dag: DagReference,
    cursor: QueryCursor,
}

/// Authenticated Dag type,
/// Overlay on top of matrix's network, 
/// uses a set, public room to send and receive the updates
/// will filter the updates signed by authenticated users from other updates
///
/// To use this type you should create (using [`AuthDag::new()`]),
/// start the sync process (with [`AuthDag::start_sync()`]), 
/// then you can query the dag [`AuthDag::query()`] and send updates to other users [`AuthDag::send_update()`]
impl AuthDag
{
    const ROOM_ID : &str = "!GXNPdYSjbFRDdXdyRK:matrix.org";
    const HOMESERVER : &str = "https://matrix.org";
    const VERSION: u8 = 3;

    /// create a new Authenticated Dag,
    /// you need to pass matrix's credentials as parameters
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

        println!("Start Syncing");
        let response = client.sync_once(Default::default()).await.unwrap();

        let oroom_id = RoomId::parse(Self::ROOM_ID).expect("failed to parse room id");
        let room = client.get_room(&oroom_id).expect("Room not found"); 

        let settings = SyncSettings::default()
            .token(response.next_batch.clone());

        tokio::spawn(async move {
            let _ = client.sync(settings.clone()).await;
        });



        Self {
            room,
            user: username.to_string(),
            dag: context,
            cursor: QueryCursor::default(),
        }
    }

    /// Send update to other users
    pub async fn send_update(&mut self, cmd: String) {
        let dag= self.dag.read().await;
        let (node,cursor) = dag.gen_node(cmd, Some(self.cursor.clone()));
        self.cursor = cursor;

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

    /// Query the Dag,
    ///
    /// provide a function `f`, that will be called for each of the nodes in the dag
    /// while keeping the order of the dag (older nodes will be called first)
    ///
    /// if a `log` is provided then the nodes that were updated before 
    /// will **not** be updated again making sure that `f` 
    /// is only called once for each node of the dag
    pub fn query(&mut self) -> Vec<String>
    {
        let (res,cursor) = tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            runtime.block_on(async {
                let map = self.dag.read().await;
                map.query(Some(self.cursor.clone()))
            })
        });
        self.cursor = cursor;
        return res
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
