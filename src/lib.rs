use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid log entry")]
    InvalidEntry,
}

pub type Result<T> = std::result::Result<T, WalError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub id: u64,
    pub timestamp: u64,
    pub data: Vec<u8>,
}

pub struct WriteAheadLog {
    file: Arc<Mutex<File>>,
    current_id: u64,
    max_size: u64,
}

impl WriteAheadLog {
    pub fn new<P: AsRef<Path>>(path: P, max_size: u64) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            current_id: 0,
            max_size,
        })
    }

    pub fn append(&mut self, data: Vec<u8>) -> Result<LogEntry> {
        let entry = LogEntry {
            id: self.current_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data,
        };

        let serialized = serde_json::to_vec(&entry)?;
        let mut file = self.file.lock().unwrap();
        file.write_all(&serialized)?;
        file.write_all(b"\n")?;
        file.flush()?;

        self.current_id += 1;
        Ok(entry)
    }

    pub fn read_all(&self) -> Result<Vec<LogEntry>> {
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start(0))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let entries: Vec<LogEntry> = contents
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_str(line))
            .collect::<std::result::Result<_, _>>()?;

        Ok(entries)
    }

    pub fn clear(&self) -> Result<()> {
        let mut file = self.file.lock().unwrap();
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }
}
