use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};
use std::path::Path;
use std::time::Instant;
use std::cmp;
use serde_json;
use sha3::Sha3_256;

use auth_crdt::{dag::MerkleDag, node::Node, QueryCursor};

pub type Dag =  MerkleDag<Vec<u8>>;

pub mod message;
pub mod logger;
use crate::message::{Message, Command}; 
use crate::logger::LogFile; 

struct Context {
    pub clock: u64,
    pub dag: Arc<RwLock<Dag>>,
    pub log_file: LogFile,
    pub cursor: QueryCursor,
}

fn handle_connection(mut stream: TcpStream,mut ctx: Context ) {
    let mut header = [0; 17];

    loop {
        if stream.read_exact(&mut header).is_err() {
            println!("Failed to read message header.");
            break;
        }

        #[cfg(feature = "debug")]
        println!("received new message");
        let mut msg = Message::header_from_bytes(&header).unwrap();

        #[cfg(feature = "debug")]
        println!("{:?}", msg);

        ctx.clock = cmp::max(ctx.clock, msg.clock);

        let mut buffer = vec![0;  msg.length as usize];
        if stream.read_exact(&mut buffer).is_err() {
            println!("Failed to read message.");
            break;
        }
        msg.message = buffer;

        let answer = process_message(&mut ctx, msg);
        let ser_answer = answer.to_bytes();

        stream.write_all(&ser_answer).unwrap();
        ctx.clock += 1;
    }
}

fn process_message(ctx: &mut Context, message: Message) -> Message {
    match message.get_command() {
        Command::Update => {
            let mut dag = ctx.dag.write().unwrap();
            let start = Instant::now();

            let layer = dag.get_top_layer();
            let parents = dag.get_heads();
            let key = "".to_string().into();
            let node = Node::new::<Sha3_256>(&key, &message.message, &parents, layer + 1);
            let res = dag.add_node(node, Some(ctx.cursor.clone()));

            match res {
                Ok(cursor) => { ctx.cursor = cursor; },
                Err(_err) => {
                    return Message::new(Command::Error, ctx.clock);
                }
            };


            let time = start.elapsed();
            ctx.log_file.log(0,"apply".to_string(),time, 0);

            Message::new(Command::Acknowledge, ctx.clock)
        }
        Command::Query => {
            let dag = ctx.dag.read().unwrap();

            let start = Instant::now();
            let (change_array, cursor) = dag.query(Some(ctx.cursor.clone()));
            ctx.cursor = cursor;

            let time = start.elapsed();
            ctx.log_file.log(0,"query".to_string(),time, 0);

            let mut ret = Message::new(Command::Acknowledge, ctx.clock);
            let json_str = serde_json::to_string(&change_array)
                .expect("Failed to serialize the array to JSON");
            ret.set_message(json_str.as_bytes().to_vec());
            ret 
        }
        Command::Acknowledge => {
            Message::error(ctx.clock, "Request is syntactically correct but Acknowledges cannot be processed.".to_string())
        }
        Command::Unknown => {
            Message::error(ctx.clock, "Request is syntactically correct but Unknowns cannot be processed.".to_string())
        }
        Command::Error => {
            Message::error(ctx.clock, "Request is syntactically correct but Errors cannot be processed.".to_string())
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()>  {
    let listener = TcpListener::bind("127.0.0.1:20076")?;
    println!("WebSocket Server running on ws://127.0.0.1:20076");
    let dag = MerkleDag::<Vec<u8>>::new();
    let dag_ref = Arc::new(RwLock::new(dag));

    loop {
        match listener.accept(){
            Err(e) => {
                println!("couldn't get client: {e:?}");
                return Err(e);
            },
            Ok((socket, addr)) => {
                println!("new client: {addr:?}");
                let ctx = Context {
                    clock: 0,
                    dag: Arc::clone(&dag_ref),
                    log_file: LogFile::new(Path::new(&format!("log_{:}_{:}.csv", 
                                addr, chrono::offset::Utc::now()))),
                    cursor: QueryCursor::default(),
                };
                tokio::spawn(async move {
                    handle_connection(socket, ctx);
                });
            },
        }
    }
}
