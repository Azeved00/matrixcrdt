use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::cmp;

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
        println!("{:?}", header);
        let mut msg = Message::header_from_bytes(&header).unwrap();

        ctx.clock = cmp::max(ctx.clock, msg.clock);

        let mut buffer = vec![0;  msg.length as usize];
        if stream.read_exact(&mut buffer).is_err() {
            println!("Failed to read message.");
            break;
        }
        msg.message = buffer;

        let answer = process_message(&mut ctx, msg);
        println!("{:?}", answer);
        let ser_answer = answer.to_bytes();
        println!("{:?}", ser_answer);

        stream.write_all(&ser_answer).unwrap();
        ctx.clock += 1;
    }
}

fn encode_changes(messages: Vec<Vec<u8>>) -> Vec<u8> {
    let mut encoded = Vec::new();

    let num_messages = messages.len() as u64;
    encoded.extend_from_slice(&num_messages.to_le_bytes());

    for message in messages {
        let length = message.len() as u64;
        encoded.extend_from_slice(&length.to_le_bytes());
        encoded.extend_from_slice(&message);
    }

    encoded
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
            let message = encode_changes(change_array);
            ret.set_message(message);
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
