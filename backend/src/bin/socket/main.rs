use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::cmp;
use std::vec::Vec;

enum Command {
    Update,
    Query,

    Error,
    Unknown,
}
struct Message {
    pub command: u8,
    pub clock:   u32,
    pub length:  u32,
    pub message: Vec<u8>,
}
impl Message {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(self.command);
        buffer.extend_from_slice(&self.clock.to_be_bytes());

        buffer.extend_from_slice(&self.length.to_be_bytes());
        buffer.extend_from_slice(&self.message);

        buffer
    }

    fn header_from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 9 {
            return Err("Data too short to be a valid message".to_string());
        }
        let command = data[0];

        let clock = u32::from_be_bytes(data[1..5].try_into().unwrap());
        let length = u32::from_be_bytes(data[5..9].try_into().unwrap());

        Ok(Self{
            command,
            clock,
            length,
            message: vec![],
        })
    }

    fn get_command(&self) -> Command {
        match  self.command {
            0 => Command::Save,
            1 => Command::Query,
            _ => Command::Unknown,
            
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut header = [0; 9];
    let mut clock:u32 = 0;

    loop {
        if stream.read_exact(&mut header).is_err() {
            println!("Failed to read message header.");
            break;
        }

        let mut msg = Message::header_from_bytes(&header).unwrap();
        clock = cmp::max(clock, msg.clock);

        let mut buffer = vec![0;  msg.length as usize];
        if stream.read_exact(&mut buffer).is_err() {
            println!("Failed to read message.");
            break;
        }
        msg.message = buffer;
        handle_message(msg);
        
        stream.write_all(b"Hello, Client!").unwrap();
        todo!("build and send return message");
    }
}

fn handle_message(message: Message) -> Message {
    match message.get_command() {
        Command::Update => {
            todo!("make update to dag");
        }
        Command::Query => {
        }
        Command::Unknown => {
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
