#![cfg(feature = "bench")] 

use std::fs::{OpenOptions,File};
use std::io::Write;
use std::time::Duration;
use std::path::Path;

#[cfg(feature = "bench")]
pub struct LogFile
{
    log_file: File,
    index: usize,
}

#[cfg(feature = "bench")]
impl LogFile {
    pub fn new(path: &Path) -> Self{

        let mut log_file = OpenOptions::new()
                .write(true)  
                .truncate(true)
                .create(true) 
                .open(path)
                .expect("Failed to open client log file");
        writeln!(log_file,"id,dag_size,operation,time,space").unwrap();

        Self {
            log_file,
            index: 0
        }
    }

    pub fn log(&mut self,size:usize, op: String, time: Duration, space:usize) {
        self.index+=1;
        writeln!(self.log_file,"{:},{:},{:},{:},{:}", self.index, size, op, time.as_micros(), space)
            .unwrap();
    }
}
