use std::{
    collections::BTreeMap,
    sync::Arc,
};
use tokio::sync::RwLock;
use tokio::runtime::Runtime;
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
use crate::StoreCommand;


// We use ruma to define our custom events. Just declare the events content
// by deriving from `EventContent` and define `ruma_events` for the metadata
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "fcup.acrdt.update", kind = MessageLike)]
pub struct UpdateEventContent {
    cmd: StoreCommand,
    author: String,
    version: u8,
    //hash: Hash,
    //parents: Vec<Hash>
}

pub struct Store {
    pub room: Room,
    pub client: Arc<RwLock<Client>>,
    pub user: String,
    pub map: Arc<RwLock<BTreeMap<u64,u64>>>,
    pub sync_settings: SyncResponse,
}

impl Store {
    const ROOM_ID : &str = "!GXNPdYSjbFRDdXdyRK:matrix.org";
    const HOMESERVER : &str = "https://matrix.org";
    const VERSION: u8 = 2;

    pub async fn new(
        username: &str,
        password: &str,
    ) -> Self {
        //create a new Crdtzza

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


        let context = MapContext::default();
        let map_ref = Arc::clone(&context.map);
        client.add_event_handler_context(context);
        client.add_event_handler(self::map_on_update);

        let response = client.sync_once(Default::default()).await.unwrap();

        let oroom_id = RoomId::parse(Self::ROOM_ID).expect("failed to parse room id");
        let room = client.get_room(&oroom_id).expect("Room not found"); 

        Store{
            room,
            client: Arc::new(RwLock::new(client)),
            user: username.to_string(),
            map: map_ref,
            sync_settings: response,
        }
    }

    pub async fn send_update(&self, cmd: StoreCommand) {
        let content = UpdateEventContent {
            cmd,
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

    pub fn query(&self, item_id: &u64) -> u64 {
        let result  = tokio::task::block_in_place(|| {
           // Create a small runtime to execute the async function
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let map = self.map.read().await;
                 map.get(item_id).unwrap_or(&0).clone()
            })
        });

        return result;
    }
}

#[derive(Debug, Default, Clone)]
pub struct MapContext{
    map: Arc<RwLock<BTreeMap<u64,u64>>>,
}
async fn map_on_update(event: SyncUpdateEvent, room: Room, mapctx: Ctx<MapContext>) {
    //println!("received update");
    if room.state() != RoomState::Joined {
        return;
    }
    
    let original = event.as_original()
        .expect("Cant get the original of received event");
    if original.content.version != Store::VERSION {
        return
    }


    let mut map = mapctx.map.write().await;
    
    //send an update
    match original.content.cmd {
        StoreCommand::Add(id, num) => {
            if let Some(x) = map.get_mut(&id) {
                *x += num ;
            } else {
                map.insert(id, num);
            }
        },
        StoreCommand::Remove(id, num) => {
            if let Some(x) = map.get_mut(&id) {
                *x -= num ;
            } else {
                map.insert(id, num);
            }
        },
        StoreCommand::Delete(id) => {
            map.remove(&id);
        },
    }
}
