use std::{
    collections::BTreeMap,
    sync::{RwLock,Arc},
};
use serde::{Deserialize, Serialize};
use matrix_sdk::{
    config::SyncSettings,
    event_handler::Ctx,
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
    ruma::{room_id, RoomId},
    Room,  RoomState,
    Client
};
use crate::{StoreCommand, Hash};


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
    pub room: Option<Room>,
    pub client: Arc<RwLock<Client>>,
    pub user: String,
    pub map: Arc<RwLock<BTreeMap<u64,u64>>>,
}

impl Store {
    const ROOM_ID : &str = "!GXNPdYSjbFRDdXdyRK:matrix.org";
    const HOMESERVER : &str = "https://matrix.org";
    const VERSION: u8 = 2;

    pub async fn new(
        username: &str,
        password: &str,
    ) -> Self {
        //create a new Crdt

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
        client.add_event_handler_context(context.clone());
        client.add_event_handler(on_update);


        Store{
            room: None,
            client: Arc::new(RwLock::new(client)),
            user: username.to_string(),
            map: context.map,
        }
    }

    pub async fn send_update(&self, cmd: StoreCommand) {
        let content = UpdateEventContent {
            cmd,
            author: self.user.clone(),
            version: Self::VERSION,
        };
        match &self.room {
            Some(r) => { 
                r.send(content).await.unwrap();
            },
            None => {}
        }
    }

    pub async fn start_sync(&self) {
        let client = self.client.write().unwrap();

        let response = client.sync_once(Default::default()).await.unwrap();

        /*
        println!("Timeline");
        println!("{:#?}", response.rooms
            .join.get(room_id!("!GXNPdYSjbFRDdXdyRK:matrix.org"))
            .unwrap()
            .timeline.events);
            */

        //println!("Getting crdt room");
        let oroom_id = RoomId::parse(Self::ROOM_ID).expect("failed to parse room id");
        let room = client.get_room(&oroom_id).expect("Room not found"); 
        drop(client);


        let settings = SyncSettings::default().token(response.next_batch);
        let client_clone = Arc::clone(&self.client);
        tokio::spawn( async move {
            let client = client_clone.write().unwrap();
            let _ = client.sync(settings.clone());
        });
    }

    pub fn query(&self, item_id: &u64) -> u64 {
        let map = self.map.read().unwrap();

        let result = map.get(item_id).unwrap_or(&0).clone();
        return result;
    }
}

#[derive(Debug, Default, Clone)]
pub struct MapContext{
    map: Arc<RwLock<BTreeMap<u64,u64>>>,
}
async fn on_update(event: SyncUpdateEvent, room: Room, mapctx: Ctx<MapContext>) {
    if room.state() != RoomState::Joined {
        return;
    }
    
    let original = event.as_original().expect("Cant get the original of received event");
    if original.content.version != Store::VERSION {
        return
    }


    let mut map = mapctx.map.write().unwrap();
    
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
