///  This is an example showcasing how to build a very simple bot with custom
/// events  using the matrix-sdk. To try it, you need a rust build setup, then
/// you can run: `cargo run  -- <homeserver_url> <user> <password>`
///
/// Use a second client to open a DM to your bot or invite them into some room.
/// You should see it automatically join. Then post `!ping`  and observe the log
/// of the bot. You will see that it sends the `Ping` event and upon receiving
/// it responds with the `Ack` event send to the room. You won't see that in
/// most regular clients, unless you activate showing of unknown events.
use std::sync::Arc;
use tokio::sync::Mutex;

use matrix_acrdt::crdt::Crdt;
use matrix_acrdt::tui::print_menu;


use std::{
    io,
    env,
    process::exit
};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // set up some simple stderr logging. You can configure it by changing the env
    // var `RUST_LOG`
    //tracing_subscriber::fmt::init();

    // parse the command line for homeserver, username and password
    let (homeserver_url, username, password) =
        match (env::args().nth(1), env::args().nth(2), env::args().nth(3)) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => {
                eprintln!(
                    "Usage: {} <homeserver_url> <username> <password>",
                    env::args().next().unwrap()
                );
                // exit if missing
                exit(1)
            }
        };


    let crdt = Crdt::login(homeserver_url, &username, &password).await;

     loop {
        print_menu();
        let mut o = Default::default();
        io::stdin().read_line(&mut o).unwrap();
        let option : u32 = o.trim().parse().unwrap();

        match option {
            1 => {
                println!("sending update");
                crdt.send_update_event().await;
            },
            2 => { break; },
            _ => { break; },
        }
    }


    Ok(())
}
