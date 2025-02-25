use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::cmp;
use serde_json;

use auth_crdt::{AuthDag, QueryCursor};

pub mod message;
use crate::message::{Message, Command}; 

struct Context {
    pub clock: u64,
    pub dag: AuthDag,
    pub cursor: QueryCursor,
    
}

fn handle_connection(mut stream: TcpStream) {
    let mut header = [0; 17];
    let mut ctx = Context {
        clock: 0,
        dag: AuthDag::new("My super secret Key".to_string().into()),
        cursor: QueryCursor::default(),
    };


    loop {
        if stream.read_exact(&mut header).is_err() {
            println!("Failed to read message header.");
            break;
        }

        println!("received new message");
        let mut msg = Message::header_from_bytes(&header).unwrap();
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
            let node = ctx.dag.gen_node(message.message, Some(ctx.cursor.clone()));
            ctx.dag.add_node(node, None);
            Message::new(Command::Acknowledge, ctx.clock)
        }
        Command::Query => {
            let (change_array, cursor) = ctx.dag.query(Some(ctx.cursor.clone()));
            ctx.cursor = cursor;
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

    loop {
        match listener.accept(){
            Err(e) => {
                println!("couldn't get client: {e:?}");
                return Err(e);
            },
            Ok((socket, addr)) => {
                println!("new client: {addr:?}");
                tokio::spawn(async move {
                    handle_connection(socket);
                });
            },
        }
    }
}
