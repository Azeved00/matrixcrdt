///  This is an example showcasing how to build a very simple bot with custom
/// events  using the matrix-sdk. To try it, you need a rust build setup, then
/// you can run: `cargo run <user> <password>`
///
/// Use a second client to open a DM to your bot or invite them into some room.
/// You should see it automatically join. Then post `!ping`  and observe the log
/// of the bot. You will see that it sends the `Ping` event and upon receiving
/// it responds with the `Ack` event send to the room. You won't see that in
/// most regular clients, unless you activate showing of unknown events.
pub mod crdt;
pub mod dag;

use dag::AuthDag;
use crdt::CRDT;

use std::{
    io,
    io::Write,
    env,
    process::exit,
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
    
    let mut store = CRDT::new(&username, &password).await;

    loop {
        println!("Choose an option:");
        println!("1. Update");
        println!("2. Save");
        println!("3. Query");
        println!("4. Pretty Print Automerge Doc");
        println!("5. Pretty Print Dag");
        println!("Enter your choice (or type 'q' to quit):");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim();
        if input.eq_ignore_ascii_case("q") {
            println!("Exiting. Goodbye!");
            break;
        }

        match input.parse::<u32>() {
            Ok(1) => {
                print!("Enter key: ");
                io::stdout().flush().unwrap();
                let mut key = String::new();
                io::stdin().read_line(&mut key).unwrap();
                let key = key.trim();

                print!("Enter value: ");
                io::stdout().flush().unwrap();
                let mut value = String::new();
                io::stdin().read_line(&mut value).unwrap();
                let value = value.trim();

                store.update(key, value);

                println!("Added or updated key '{}'.", key);
            }
            Ok(2) => {
                store.save().await;
            }
            Ok(3) => {
                store.query();
            }
            Ok(4) => {
                let s = store.pretty_print_doc();
                println!("{}",s);
            }
            Ok(5) => {
                let s = store.pretty_print_dag();
                println!("{}",s);
            }
            Ok(_) => {
                println!("Invalid option. Please enter 1, 2, 3 or 4.");
            }
            Err(_) => {
                println!("Invalid input. Please enter a number.");
            }
        }


    }
    Ok(())
}
