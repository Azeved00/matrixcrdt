use std::sync::Arc;
use sha3::Sha3_256;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use matrix_sdk::{
    config::SyncSettings,
    event_handler::Ctx,
    ruma::events::macros::EventContent,
    Room,  RoomState,
    Client,
    ruma::api::client::room::Visibility,
    ruma::api::client::room::create_room::v3::{
        RoomPreset,
        Request as CreateRoomRequest,
    }
};

use auth_crdt::{auth_dag::AuthMerkleDag, auth_node::AuthNode, QueryCursor};

type Data = Vec<u8>;
type DagReference = Arc<RwLock<AuthMerkleDag<Sha3_256, Data>>>; 


#[derive(Clone, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "fcup.acrdt.update", kind = MessageLike)]
struct UpdateEventContent
{
    cmd: AuthNode<Data>,
    author: String,
    version: u8,
}

#[derive(Clone)]
pub struct AuthMatrixDag
{
    room: Room,
    user: String,
    dag: DagReference,
}

/// Authenticated Dag type,
/// Overlay on top of matrix's network, 
/// uses a set, public room to send and receive the updates
/// will filter the updates signed by authenticated users from other updates
///
/// To use this type you should create (using [`AuthMatrixDag::new()`]),
/// start the sync process (with [`AuthMatrixDag::start_sync()`]), 
/// then you can query the dag [`AuthMatrixDag::query()`] and
/// send updates to other users [`AuthMatrixDag::send_update()`]
impl AuthMatrixDag
{
    const HOMESERVER : &str = "https://matrix.org";
    const VERSION: u8 = 4;

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

        println!("Initial Sync Step");
        let response = client.sync_once(Default::default()).await.unwrap();
        let settings = SyncSettings::default()
            .token(response.next_batch.clone());

        println!("Create Room");
        let mut request = CreateRoomRequest::new();
        request.name= Some("My Benchmark Room".into());
        request.topic= Some("Room for benchmarking tests".into());
        request.preset= Some(RoomPreset::PublicChat);
        request.is_direct= false;
        request.visibility= Visibility::Public;

        let room = client.create_room(request).await.unwrap();

        println!("Start Syncing Thread");
        tokio::spawn(async move {
            let _ = client.sync(settings.clone()).await;
        });

        Self {
            room,
            user: username.to_string(),
            dag: context,
        }
    }

    /// Send update to other users
    pub async fn send_update(&mut self, cmd: Data, c: Option<QueryCursor>) 
        -> std::io::Result<QueryCursor> 
    {
        let mut dag= self.dag.write().await;
        let (node,cursor)= dag.insert(cmd,c)
            .expect("failed to add node to DAG");

        let content = UpdateEventContent {
            cmd: node.clone(),
            author: self.user.clone(),
            version: Self::VERSION,
        };

        let result = self.room.send(content).await;
        if result.is_err() {
            panic!("An error occurred: {:?}", result.unwrap_err());
        }
        return Ok(cursor)
    }

    /// Query the Dag,
    ///
    /// provide a function `f`, that will be called for each of the nodes in the dag
    /// while keeping the order of the dag (older nodes will be called first)
    ///
    /// if a `log` is provided then the nodes that were updated before 
    /// will **not** be updated again making sure that `f` 
    /// is only called once for each node of the dag
    pub fn query(&self, cursor: Option<QueryCursor>) -> (Vec<Data>, QueryCursor)
    {
        let (res,cursor) = tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            runtime.block_on(async {
                let map = self.dag.read().await;
                map.query(cursor.clone())
            })
        });
        return (res, cursor)
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
    if original.content.version != AuthMatrixDag::VERSION {
        return
    }

    let mut dag = ctx.write().await;
    let _ = dag.insert_node(original.content.cmd.clone(), None);
}
