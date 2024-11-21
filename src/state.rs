use std::{
    collections::BTreeMap,
    sync::Arc,
};
use matrix_sdk::{
    //config::SyncSettings,
    event_handler::Ctx,
    ruma::events::{
        macros::EventContent,
        room::{
//            member::StrippedRoomMemberEvent,
            //message::{MessageType, OriginalSyncRoomMessageEvent},
        },
    },
    //ruma::{room_id, RoomId ,RoomAliasId},
    Client, Room,  RoomState,
};
use serde::{Deserialize, Serialize};
use crate::crdt::Crdt;

pub struct Store {
    map: BTreeMap<u64, u64>,
    store_id: u8,
    network: Crdt,
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
#[ruma_event(type = "rs.matrix-sdk.example.ack", kind = MessageLike)]
pub struct UpdateEventContent {
    cmd: StoreCommand,
    author: String,
}
#[derive(Debug, Default, Clone)]
pub struct CustomContext {
    ping_counter: Arc<BTreeMap<u64,u64>>,
}

impl Store {
    const ROOM_ID : &str = "!GXNPdYSjbFRDdXdyRK:matrix.org";
    const HOMESERVER : &str = "https://matrix.org";

    pub async fn new(
        username: &str,
        password: &str,
    ) -> Self {
        //create a new Crdt
        let network =Crdt::login(
            Self::HOMESERVER.to_string(), 
            Self::ROOM_ID, 
            username, 
            password).await;

        network.room.add_event_handler(on_ping_event);

        Store{
            map: BTreeMap::new(),
            store_id: 1,
            network,
        }
    }

    pub async fn update(&mut self, cmd: StoreCommand) {
        let content = UpdateEventContent {
            cmd,
            author: "snow".to_string(),
        };
        self.network.room.send(content).await.unwrap();
    }

    pub fn on_update(&mut self, cmd: StoreCommand) {
        //send an update
        match cmd {
            StoreCommand::Add(id, num) => {
                if let Some(x) = self.map.get_mut(&id) {
                    *x += num ;
                } else {
                    self.map.insert(id, num);
                }
            },
            StoreCommand::Remove(id, num) => {
                if let Some(x) = self.map.get_mut(&id) {
                    *x -= num ;
                } else {
                    self.map.insert(id, num);
                }
            },
            StoreCommand::Delete(id) => {
                self.map.remove(&id);
            },
        }
    }
}

async fn on_ping_event(event: SyncUpdateEvent, room: Room) {
    if room.state() != RoomState::Joined {
        return;
    }

    println!("update received {:}", event.author);
}
