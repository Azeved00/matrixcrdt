//use std::sync::{
//    atomic::{AtomicU64, Ordering},
//    Arc,
//};

use matrix_sdk::{
    config::SyncSettings,
    //event_handler::Ctx,
    //ruma::events::{
        //macros::EventContent,
        //room::{
//            member::StrippedRoomMemberEvent,
            //message::{MessageType, OriginalSyncRoomMessageEvent},
        //},
    //},
    ruma::RoomId,
    Client, Room,
};
//use tokio::time::{sleep, Duration};



pub struct Crdt {
    pub room: Room,
    pub user: String,
    //pub client: Client,
    //pub sync_settings: SyncSettings,
}

impl Crdt {


    pub async fn login(
        homeserver_url: String,
        room_id: &str,
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

        let oroom_id = RoomId::parse(room_id).expect("failed to parse room id");
        let room = client.get_room(&oroom_id).expect("Room not found"); 


        tokio::spawn( async move {
            let _ = client.sync(settings.clone()).await;
        });


        return Crdt{
            user: username.to_string(),
            room, //client, sync_settings: settings
        };
    }
    //pub async fn sync(&self) -> anyhow::Result<()> {
    //    self.client.sync(self.sync_settings.clone()).await?;
    //
    //    Ok(())
    //}

}
