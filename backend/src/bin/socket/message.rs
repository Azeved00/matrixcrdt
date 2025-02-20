use std::vec::Vec;

pub enum Command {
    Update,
    Query,

    Error,
    Unknown,
}
pub struct Message {
    pub command: u8,
    pub clock:   u32,
    pub length:  u32,
    pub message: Vec<u8>,
}

impl Message {
    pub fn new(cmd: Command, clock:u32) -> Self{
        let command = match cmd {
            Command::Update => 0,
            Command::Query => 1,
            Command::Error => 2,
            Command::Unknown => 3,
        };
        Self{
            command,
            length: 0,
            clock,
            message: vec![],
        }
    }

    pub fn set_message(&mut self, msg:Vec<u8>){
        self.length = msg.len() as u32;
        self.message = msg;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(self.command);
        buffer.extend_from_slice(&self.clock.to_be_bytes());

        buffer.extend_from_slice(&self.length.to_be_bytes());
        buffer.extend_from_slice(&self.message);

        buffer
    }

    pub fn header_from_bytes(data: &[u8]) -> Result<Self, String> {
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

    pub fn get_command(&self) -> Command {
        match  self.command {
            0 => Command::Update,
            1 => Command::Query,
            2 => Command::Query,
            _ => Command::Unknown,
        }
    }
}

