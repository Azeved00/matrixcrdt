use std::net::{TcpListener, TcpStream};
#[cfg(feature = "debug")]
use std::net::SocketAddr;
use std::io::{Read, Write};
#[cfg(feature = "bench")]
use std::time::Instant;
use std::cmp;
use serde_json;

pub mod dag;

#[cfg(feature = "bench")]
use auth_crdt::common::logger::LogFile;
use auth_crdt::QueryCursor;
use auth_crdt::common::message::Message;
use auth_crdt::common::message::Command;
use crate::dag::AuthMatrixDag;


struct Context {
    pub clock: u64,
    pub dag: AuthMatrixDag,
#[cfg(feature = "debug")]
    pub addr: SocketAddr,
#[cfg(feature = "bench")]
    pub log_file: LogFile,
    pub cursor: QueryCursor,
}

async fn handle_connection(mut stream: TcpStream,mut ctx: Context ) {
    let mut header = [0; 17];

    loop {
        if stream.read_exact(&mut header).is_err() {
            println!("Failed to read message header.");
            break;
        }

        let mut msg = Message::header_from_bytes(&header).unwrap();

        ctx.clock = cmp::max(ctx.clock, msg.clock);

        let mut buffer = vec![0;  msg.length as usize];
        if stream.read_exact(&mut buffer).is_err() {
            println!("Failed to read message.");
            break;
        }
        msg.message = buffer;
#[cfg(feature = "debug")]
        println!("{:} Received: {:?}",ctx.addr.port() ,msg);

        let answer = process_message(&mut ctx, msg).await;
#[cfg(feature = "debug")]
        println!("{:} Answered: {:?}",ctx.addr.port(), answer);
        let ser_answer = answer.to_bytes();

        stream.write_all(&ser_answer).unwrap();
        ctx.clock += 1;
    }
}

async fn process_message(ctx: &mut Context, message: Message) -> Message {
    match message.get_command() {
        Command::Update => {
#[cfg(feature = "bench")]
            let start = Instant::now();
            let res = ctx.dag.send_update(message.message, Some(ctx.cursor.clone()))
                .await;
#[cfg(feature = "debug")]
            println!("{:?}", ctx.dag.len());
            match res {
                Ok(cursor) => {ctx.cursor = cursor;},
                Err(_err) => {
#[cfg(feature = "debug")]
                    println!("Error when updating: {}", _err);
                    return Message::new(Command::Error, ctx.clock);
                }
            };

#[cfg(feature = "debug")]
            println!("{:?}", ctx.cursor);

#[cfg(feature = "bench")]
            let time = start.elapsed();
#[cfg(feature = "bench")]
            ctx.log_file.log(0,"apply".to_string(),time, 0);

            Message::new(Command::Acknowledge, ctx.clock)
        }
        Command::StatefulQuery => {
#[cfg(feature = "bench")]
            let start = Instant::now();
#[cfg(feature = "debug")]
            println!("stateful query");
#[cfg(feature = "debug")]
            println!("{:?}", ctx.cursor);
#[cfg(feature = "debug")]
            println!("{:?}", ctx.dag.get_dag().get_heads());

            let (change_array, cursor) = ctx.dag.query(Some(ctx.cursor.clone()));
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
#[cfg(feature = "debug")]
            println!("stateless query");

#[cfg(feature = "bench")]
            let start = Instant::now();
            let (change_array, cursor) = ctx.dag.query(None);
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
        let _ = run_server().await;
    });
    Ok(())
}

async fn run_server() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:20076")?;
    println!("WebSocket Server running on ws://127.0.0.1:20076");
    let dag = AuthMatrixDag::new("Random username go", "My super secret Key").await;

    loop {
        match listener.accept() {
            Err(e) => {
                println!("couldn't get client: {e:?}");
                return Err(e);
            },
            Ok((socket, _addr)) => {
#[cfg(feature = "debug")]
                println!("new client: {_addr:?}");
                let ctx = Context {
                    clock: 0,
                    dag: dag.clone(),
#[cfg(feature = "debug")]
                    addr: _addr,
#[cfg(feature = "bench")]
                    log_file: LogFile::new(std::path::Path::new(&format!("backend/log_{:}.csv", _addr.port()))),
                    cursor: QueryCursor::default(),
                };
                tokio::spawn(async move {
                    handle_connection(socket, ctx).await;
                });
            },
        }
    }
}
