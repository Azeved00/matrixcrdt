///  This is an example showcasing how to build a very simple bot with custom
/// events  using the matrix-sdk. To try it, you need a rust build setup, then
/// you can run: `cargo run  -- <homeserver_url> <user> <password>`
///
/// Use a second client to open a DM to your bot or invite them into some room.
/// You should see it automatically join. Then post `!ping`  and observe the log
/// of the bot. You will see that it sends the `Ping` event and upon receiving
/// it responds with the `Ack` event send to the room. You won't see that in
/// most regular clients, unless you activate showing of unknown events.
use matrix_acrdt::{
    StoreCommand,
    store_crdt::Store,
    tui::TerminalUI
};


use std::{
    io,
    env,
    process::exit
};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //tracing_subscriber::fmt::init();

    // parse the command line for homeserver, username and password
    let (username, password) =
        match (env::args().nth(1), env::args().nth(2)) {
            (Some(a), Some(b)) => (a, b),
            _ => {
                eprintln!(
                    "Usage: {} <username> <password>",
                    env::args().next().unwrap()
                );
                // exit if missing
                exit(1)
            }
        };

    let mut store = Store::new(&username, &password).await;
    let mut ui = TerminalUI::new(store)?;
    let _ = ui.run();


    Ok(())
}
