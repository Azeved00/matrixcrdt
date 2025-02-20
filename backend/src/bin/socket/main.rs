use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::cmp;

use auth_crdt::{AuthDag, QueryCursor};

pub mod message;
use crate::message::{Message, Command}; 

struct Context {
    pub clock: u32,
    pub dag: AuthDag,
    pub cursor: QueryCursor,
    
}

fn handle_connection(mut stream: TcpStream) {
    let mut header = [0; 9];
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

        let mut msg = Message::header_from_bytes(&header).unwrap();
        ctx.clock = cmp::max(ctx.clock, msg.clock);

        let mut buffer = vec![0;  msg.length as usize];
        if stream.read_exact(&mut buffer).is_err() {
            println!("Failed to read message.");
            break;
        }
        msg.message = buffer;
        let answer = handle_message(ctx, msg);
        let ser_answer = answer.to_bytes();

        stream.write_all(&ser_answer).unwrap();
    }
}

fn handle_message(mut ctx: Context, message: Message) -> Message {
    match message.get_command() {
        Command::Update => {
            let node = ctx.dag.gen_node(message.message, Some(ctx.cursor));
            ctx.dag.add_node(node, None);
            Message::new(Command::Update, ctx.clock+1)
        }
        Command::Query => {
            let (change_array, cursor) = ctx.dag.query(Some(ctx.cursor));
            ctx.cursor = cursor;
            let ret = Message::new(Command::Query, ctx.clock+1);
            todo!("serialize and return change_array");
            ret 
        }
        Command::Unknown => {
            Message::new(Command::Error, ctx.clock + 1)
        }
        Command::Error => {
            Message::new(Command::Error, ctx.clock+1)
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
