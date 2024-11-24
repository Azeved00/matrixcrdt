use std::{
    collections::BTreeMap,
    sync::{RwLock,Arc},
    io::stdin,
};
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
use serde::{Deserialize, Serialize};

pub struct Store {
    pub room: Room,
    pub user: String,
    pub map: Arc<RwLock<BTreeMap<u64,u64>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum StoreCommand{
    Add(u64, u64),
    Remove(u64, u64),
    Delete(u64),
}



// We use ruma to define our custom events. Just declare the events content
// by deriving from `EventContent` and define `ruma_events` for the metadata
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "fcup.acrdt.update", kind = MessageLike)]
pub struct UpdateEventContent {
    cmd: StoreCommand,
    author: String,
    version: u8,
}

#[derive(Debug, Default, Clone)]
pub struct MapContext{
    map: Arc<RwLock<BTreeMap<u64,u64>>>,
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


        println!("First Sync");
        let response = client.sync_once(Default::default()).await.unwrap();

        println!("Timeline");
        println!("{:#?}", response.rooms
            .join.get(room_id!("!GXNPdYSjbFRDdXdyRK:matrix.org"))
            .unwrap()
            .timeline.events);
        let settings = SyncSettings::default().token(response.next_batch);

        println!("Getting crdt room");
        let oroom_id = RoomId::parse(Self::ROOM_ID).expect("failed to parse room id");
        let room = client.get_room(&oroom_id).expect("Room not found"); 


        tokio::spawn( async move {
            let _ = client.sync(settings.clone()).await;
        });

        Store{
            room,
            user: username.to_string(),
            map: context.map,
        }
    }

    pub async fn send_update(&mut self, cmd: StoreCommand) {
        let content = UpdateEventContent {
            cmd,
            author: self.user.clone(),
            version: Self::VERSION,
        };
        self.room.send(content).await.unwrap();
    }

    pub fn query(&mut self, item_id: &u64) -> u64 {
        let map = self.map.read().unwrap();

        let result = map.get(item_id).unwrap_or(&0).clone();
        return result;
    }
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

    println!("update received {:}", original.content.author);
}
