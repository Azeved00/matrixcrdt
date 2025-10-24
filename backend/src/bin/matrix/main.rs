use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use std::io::{Read, Write};
#[cfg(feature = "bench")]
use std::time::Instant;
use std::cmp;
use serde_json;

use tracing_subscriber::filter::EnvFilter;
use tracing::{info, error, info_span,debug};

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
    pub addr: SocketAddr,
#[cfg(feature = "bench")]
    pub log_file: LogFile,
    pub cursor: QueryCursor,
}

async fn handle_connection(mut stream: TcpStream, mut ctx: Context) {
    let mut header = [0; 17];

    loop {
        if stream.read_exact(&mut header).await.is_err() {
            println!("Failed to read message header.");
            break;
        }

        let mut msg = Message::header_from_bytes(&header).unwrap();
        ctx.clock = cmp::max(ctx.clock, msg.clock);
        let mut buffer = vec![0; msg.length as usize];

        if stream.read_exact(&mut buffer).await.is_err() {
            println!("Failed to read message.");
            break;
        }

        msg.message = buffer;
        info!("{:} Received: {:?}", ctx.addr.port(), msg);

        let answer = process_message(&mut ctx, msg).await;
        info!("{:} Answered: {:?}", ctx.addr.port(), answer);
        let ser_answer = answer.to_bytes();

        stream.write_all(&ser_answer).await.unwrap();
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
            info!("Sending update");
            
            match res {
                Ok(cursor) => {ctx.cursor = cursor;},
                Err(_err) => {
                    error!("Error when updating: {}", _err);
                    return Message::new(Command::Error, ctx.clock);
                }
            };

            debug!("{:?}", ctx.cursor);

#[cfg(feature = "bench")]
            let time = start.elapsed();
#[cfg(feature = "bench")]
            ctx.log_file.log(0,"apply".to_string(),time, 0);

            Message::new(Command::Acknowledge, ctx.clock)
        }
        Command::StatefulQuery => {
#[cfg(feature = "bench")]
            let start = Instant::now();
            info!("stateful query");
            debug!("{:?}", ctx.cursor);

            let (change_array, cursor) = ctx.dag.query(Some(ctx.cursor.clone())).await;
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
            info!("stateless query");

#[cfg(feature = "bench")]
            let start = Instant::now();
            let (change_array, cursor) = ctx.dag.query(None).await;
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

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <port>", args[0]);
        std::process::exit(1);
    }

    let port: u16 = args[1].parse()?;

    runtime.block_on(async {
        let _ = run_server(port).await;
    });
    Ok(())
}


async fn run_server(port: u16) -> std::io::Result<()>{
    tracing_subscriber::fmt()
        //.compact()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_level(true)
        .init();

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(addr).await?;
    info!("WebSocket Server running on ws://127.0.0.1:{}", port);
    let dag = AuthMatrixDag::new("alice", "test123").await;


    loop {
        match listener.accept().await {
            Err(e) => {
                println!("couldn't get client: {e:?}");
                return Err(e);
            },
            Ok((socket, addr)) => {
                info!("new client: {addr:?}");
                let ctx = Context {
                    clock: 0,
                    dag: dag.clone(),
                    addr: addr,
#[cfg(feature = "bench")]
                    log_file: LogFile::new(std::path::Path::new(&format!("backend/log_{:}.csv", addr.port()))),
                    cursor: QueryCursor::default(),
                };
                tokio::spawn(async move {
                    handle_connection(socket, ctx).await;
                });
            },
        }
    }
}
