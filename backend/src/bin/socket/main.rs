use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};
#[cfg(feature = "bench")]
use std::time::Instant;
use std::cmp;
use serde_json;

use auth_crdt::{AuthDag, QueryCursor};
#[cfg(feature = "bench")]
use auth_crdt::common::logger::LogFile;
use auth_crdt::common::message::{Message, Command}; 


struct Context {
    pub clock: u64,
    pub dag: Arc<RwLock<AuthDag>>,
#[cfg(feature = "bench")]
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

        ctx.clock = cmp::max(ctx.clock, msg.clock);

        let mut buffer = vec![0;  msg.length as usize];
        if stream.read_exact(&mut buffer).is_err() {
            println!("Failed to read message.");
            break;
        }
        msg.message = buffer;
#[cfg(feature = "debug")]
        println!("Received: {:?}", msg);

        let answer = process_message(&mut ctx, msg);
        println!("Answered: {:?}", answer);
        let ser_answer = answer.to_bytes();

        stream.write_all(&ser_answer).unwrap();
        ctx.clock += 1;
    }
}

fn process_message(ctx: &mut Context, message: Message) -> Message {
    match message.get_command() {
        Command::Update => {
            let mut dag = ctx.dag.write().unwrap();
#[cfg(feature = "bench")]
            let start = Instant::now();

            let node = dag.gen_node(message.message, Some(ctx.cursor.clone()));
            let res = dag.add_node(node, Some(ctx.cursor.clone()));
            println!("{:?}", dag.len());
            match res {
                Ok(cursor) => {ctx.cursor = cursor;},
                Err(err) => {
#[cfg(feature = "debug")]
                    println!("Error when updating: {}", err);
                    return Message::new(Command::Error, ctx.clock);
                }
            };

            println!("{:?}", ctx.cursor);

#[cfg(feature = "bench")]
            let time = start.elapsed();
#[cfg(feature = "bench")]
            ctx.log_file.log(0,"apply".to_string(),time, 0);

            Message::new(Command::Acknowledge, ctx.clock)
        }
        Command::StatefulQuery => {
            let dag = ctx.dag.read().unwrap();

#[cfg(feature = "bench")]
            let start = Instant::now();
            println!("{:?}", ctx.cursor);
            println!("{:?}", dag.len());
            println!("{:?}", dag.linearize());
            let (change_array, cursor) = dag.query(Some(ctx.cursor.clone()));
#[cfg(feature = "debug")]
            println!("{:?}",change_array);
            ctx.cursor = cursor;

#[cfg(feature = "bench")]
            let time = start.elapsed();
#[cfg(feature = "bench")]
            ctx.log_file.log(0,"stateful_query".to_string(),time, 0);

            let mut ret = Message::new(Command::Acknowledge, ctx.clock);
            let json_str = serde_json::to_string(&change_array)
                .expect("Failed to serialize the array to JSON");
            ret.set_message(json_str.as_bytes().to_vec());
            ret 
        }
        Command::StatelessQuery => {
            let dag = ctx.dag.read().unwrap();

#[cfg(feature = "bench")]
            let start = Instant::now();
            let (change_array, cursor) = dag.query(None);
            ctx.cursor = cursor;

#[cfg(feature = "bench")]
            let time = start.elapsed();
#[cfg(feature = "bench")]
            ctx.log_file.log(0,"stateless_query".to_string(),time, 0);

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(32)
        .enable_all()
        .build()?;

    runtime.block_on( async {
        let _ = run_server();
    });
    Ok(())
}

fn run_server() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:20076")?;
    println!("WebSocket Server running on ws://127.0.0.1:20076");
    let dag = AuthDag::new("My super secret Key".to_string().into());
    let dag_ref = Arc::new(RwLock::new(dag));

    loop {
        match listener.accept() {
            Err(e) => {
                println!("couldn't get client: {e:?}");
                return Err(e);
            },
            Ok((socket, addr)) => {
                println!("new client: {addr:?}");
                let ctx = Context {
                    clock: 0,
                    dag: Arc::clone(&dag_ref),
#[cfg(feature = "bench")]
                    log_file: LogFile::new(std::path::Path::new(&format!("backend/log_{:}.csv", addr))),
                    cursor: QueryCursor::default(),
                };
                tokio::spawn(async move {
                    handle_connection(socket, ctx);
                });
            },
        }
    }
}
