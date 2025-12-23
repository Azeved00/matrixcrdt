use std::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};
#[cfg(feature = "bench")]
use std::time::Instant;
use std::cmp;
use serde_json;

use auth_crdt::{AuthDag, QueryCursor};
use auth_crdt::AuthClient;
use auth_crdt::AuthReplica;
#[cfg(feature = "bench")]
use auth_crdt::common::logger::LogFile;
use auth_crdt::common::message::{Message, Command}; 
use tracing_subscriber::filter::EnvFilter;
use tracing::{info, debug, error, info_span};


struct Context {
    pub clock: u64,
    pub dag: Arc<RwLock<AuthClient>>,
    pub addr: SocketAddr,
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

        let mut msg = Message::header_from_bytes(&header).unwrap();

        ctx.clock = cmp::max(ctx.clock, msg.clock);

        let mut buffer = vec![0;  msg.length as usize];
        if stream.read_exact(&mut buffer).is_err() {
            error!("Failed to read message.");
            break;
        }
        msg.message = buffer;
        let span = info_span!("Message", port = %ctx.addr.port(), clock=%msg.clock);
        let _enter = span.enter();

        debug!("Received: {:?}",msg);

        let answer = process_message(&mut ctx, msg);

        debug!("Answered: {:?}", answer);
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
            let res = dag.insert(message.message, Some(ctx.cursor.clone()));
            debug!("Dag Length: {:?}", dag.len());

            match res {
                Ok((_, cursor)) => {ctx.cursor = cursor;},
                Err(_err) => {
                    error!("Error when updating: {}", _err);
                    return Message::new(Command::Error, ctx.clock);
                }
            };

            info!("Cursor {:?}", ctx.cursor);

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
            debug!("Cursor {:?}", ctx.cursor);
            debug!("Heads: {:?}", dag.get_dag().get_heads());

            let (change_array, cursor) = dag.query(Some(ctx.cursor.clone()));

            debug!("Changes: {:?}", change_array.len());
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
            let (change_array, _cursor) = dag.query(Some(ctx.cursor.clone()));
            //ctx.cursor = cursor;

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
    tracing_subscriber::fmt()
        //.compact()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_level(true)
        .init();

    let listener = TcpListener::bind("127.0.0.1:20076")?;

    info!("WebSocket Server running on ws://127.0.0.1:20076");
    let dag = AuthDag::new("My super secret Key".to_string().into());
    let dag_ref = Arc::new(RwLock::new(dag));


    loop {
        match listener.accept() {
            Err(e) => {
                println!("couldn't get client: {e:?}");
                return Err(e);
            },
            Ok((socket, addr)) => {
                info!("new client: {addr:?}");
                let ctx = Context {
                    clock: 0,
                    dag: Arc::clone(&dag_ref),
                    addr,
#[cfg(feature = "bench")]
                    log_file: LogFile::new(std::path::Path::new(&format!("backend/log_{:}.csv", addr.port()))),
                    cursor: QueryCursor::default(),
                };
                tokio::spawn(async move {
                    handle_connection(socket, ctx);
                });
            },
        }
    }
}
