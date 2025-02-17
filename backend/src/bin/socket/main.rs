use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::cmp;

enum Command {
    Save,
    Query,
    Unknown,
}

fn handle_connection(mut stream: TcpStream) {
    let mut header = [0; 9];
    let mut clock:u32 = 0;

    loop {
        if stream.read_exact(&mut header).is_err() {
            println!("Failed to read message header.");
            break;
        }

        let command_code = header[0];
        let command = match command_code {
            0 => Command::Save,
            1 => Command::Query,
            _ => Command::Unknown,
            
        };
        let msg_clock = u32::from_be_bytes(header[1..5].try_into().unwrap());
        clock = cmp::max(msg_clock,clock);

        let msg_length = u32::from_be_bytes(header[5..9].try_into().unwrap());

        let mut message = vec![0; msg_length as usize];
        if stream.read_exact(&mut message).is_err() {
            println!("Failed to read message.");
            break;
        }
        handle_message(command, message);
        
        stream.write_all(b"Hello, Client!").unwrap();
        todo!("build and send return message");
    }
}

fn handle_message(cmd: Command, message: Vec<u8>) {}

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
