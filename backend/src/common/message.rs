use std::vec::Vec;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Command {
    Acknowledge=0,
    Error = 1,

    Update =2,
    StatefulQuery =3 ,
    StatelessQuery = 4,

    Unknown = 255,
}
impl Command{
    pub fn from_u8(x: u8) -> Self{
        match x {
            0 => Command::Acknowledge,
            1 => Command::Error,
            2 => Command::Update,
            3 => Command::StatefulQuery,
            4 => Command::StatelessQuery,
            _ => Command::Unknown
        }
    }
}
impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Command::Acknowledge => "Acknowledge",
            Command::Error => "Error",
            Command::Update => "Update",
            Command::StatefulQuery => "StatefulQuery",
            Command::StatelessQuery => "StatelessQuery",
            Command::Unknown => "Unknown",
        };
        write!(f, "{}", name)
    }
}


pub struct Message {
    pub command: u8,
    pub clock:   u64,
    pub length:  u64,
    pub message: Vec<u8>,
}

impl Message {
    pub fn new(cmd: Command, clock:u64) -> Self{
        let command =  cmd as u8;
        Self{
            command,
            length: 0,
            clock,
            message: vec![],
        }
    }

    pub fn error(clock: u64, msg: String) -> Self {
        let message = msg.into_bytes();
        let length = message.len() as u64;
        Self{
            command: Command::Error as u8,
            length,
            clock,
            message,
        }
    }


    pub fn set_message(&mut self, msg:Vec<u8>){
        self.length = msg.len() as u64;
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
        if data.len() < 17 {
            return Err("Data too short to be a valid message".to_string());
        }

        let command = data[0];
        let slice : [u8; 8] = data[1..9].try_into().expect("failed to read clock bytes");
        let clock = u64::from_be_bytes(slice);

        let slice : [u8; 8] = data[9..17].try_into().expect("failed to read length bytes");
        let length = u64::from_be_bytes(slice);

        Ok(Self{
            command,
            clock,
            length,
            message: vec![],
        })
    }

    pub fn get_command(&self) -> Command {
        Command::from_u8(self.command.clone())
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg_preview = if self.message.len() > 5 {
            &self.message[..5]
        } else {
            &self.message
        };

        write!(
            f,
            "Message {{ command: {}({}), clock: {}, length: {}, message: {:?} }}",
            Command::from_u8(self.command.clone()), 
            self.command,
            self.clock,
            self.length,
            msg_preview
        )
    }
}
