use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use matrix_sdk::{
    config::SyncSettings,
    //event_handler::Ctx,
    ruma::events::{
        macros::EventContent,
        room::{
//            member::StrippedRoomMemberEvent,
            message::{MessageType, OriginalSyncRoomMessageEvent},
        },
    },
    ruma::{room_id, RoomId ,RoomAliasId},
    Client, Room,  RoomState,
};
use serde::{Deserialize, Serialize};
//use tokio::time::{sleep, Duration};



pub struct Crdt {
    pub room: Room,
    //pub client: Client,
    //pub sync_settings: SyncSettings,
}

// We use ruma to define our custom events. Just declare the events content
// by deriving from `EventContent` and define `ruma_events` for the metadata
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "rs.matrix-sdk.example.ack", kind = MessageLike)]
pub struct UpdateEventContent {
    ping_id: u64,
}

async fn on_ping_event(_event: SyncUpdateEvent, room: Room) {
    if room.state() != RoomState::Joined {
        return;
    }

    //let ping_number = context.ping_counter.fetch_add(1, Ordering::SeqCst);

    //let content = UpdateEventContent { ping_id: ping_number };
    println!("Update here");
    //room.send(content).await.unwrap();
}

impl Crdt {

    const ROOM_ID : &str = "!GXNPdYSjbFRDdXdyRK:matrix.org";

    pub async fn login(
        homeserver_url: String,
        username: &str,
        password: &str,
    ) -> Self {
        println!("logging in");
        let client = Client::builder().homeserver_url(homeserver_url).build().await.expect("failed to connect to homeserver");

        client
            .matrix_auth()
            .login_username(username, password)
            .initial_device_display_name("getting started bot")
            .await
            .expect("authenticated failed");

        println!("logged in as {username}");

        println!("syncing once");
        let response = client.sync_once(Default::default()).await.unwrap();
        let settings = SyncSettings::default().token(response.next_batch);

        let room_id = RoomId::parse(Self::ROOM_ID).expect("failed to parse room id");
        let room = client.get_room(&room_id).expect("Room not found"); 
        room.add_event_handler(on_ping_event);


        tokio::spawn( async move {
            let _ = client.sync(settings.clone()).await;
        });


        return Crdt{
            room, //client, sync_settings: settings
        };
    }

    pub async fn send_update_event(&self){
        let content = UpdateEventContent { ping_id: 0};
        self.room.send(content).await.unwrap();
    }

    //pub async fn sync(&self) -> anyhow::Result<()> {
    //    self.client.sync(self.sync_settings.clone()).await?;
    //
    //    Ok(())
    //}

}
