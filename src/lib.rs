use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Errors that can occur when working with the Write-Ahead Log.
#[derive(Error, Debug)]
pub enum WalError {
    /// An I/O error occurred while reading from or writing to the log file.
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    /// A serialization or deserialization error occurred.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// The log entry is invalid or corrupted.
    #[error("invalid log entry")]
    InvalidEntry,
}

/// A type alias for Results that can contain a `WalError`.
pub type Result<T> = std::result::Result<T, WalError>;

/// Represents a single entry in the Write-Ahead Log.
///
/// Each entry contains a unique identifier, timestamp, and binary data.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    /// Unique identifier for the log entry.
    pub id: u64,
    /// Unix timestamp when the entry was created.
    pub timestamp: u64,
    /// Binary data stored in the log entry.
    pub data: Vec<u8>,
}

/// A thread-safe Write-Ahead Log implementation.
///
/// The Write-Ahead Log ensures data durability by writing entries to disk
/// before processing them. This is useful for software agents and other systems
/// that need to guarantee temporary data persistence.
///
/// # Examples
///
/// ```rust
/// use waly::{WriteAheadLog, Result};
///
/// fn main() -> Result<()> {
///     let mut wal = WriteAheadLog::new("logs.wal")?;
///     
///     // Append data to the log
///     let entry = wal.append(b"Hello, World!".to_vec())?;
///     println!("Appended entry with ID: {}", entry.id);
///     
///     // Read all entries
///     let entries = wal.read_all()?;
///     for entry in entries {
///         println!("Entry {}: {:?}", entry.id, entry.data);
///     }
///     
///     Ok(())
/// }
/// ```
pub struct WriteAheadLog {
    file: Arc<Mutex<File>>,
    current_id: u64,
}

impl WriteAheadLog {
    /// Creates a new Write-Ahead Log at the specified path.
    ///
    /// If the file doesn't exist, it will be created. If it exists, the log
    /// will be opened and the next ID will be determined from existing entries.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path where the log will be stored.
    ///
    /// # Returns
    ///
    /// Returns a `Result<WriteAheadLog>` containing the initialized log or an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waly::WriteAheadLog;
    ///
    /// let wal = WriteAheadLog::new("logs.wal")?;
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(path)?;

        let current_id = Self::get_new_id(&file)?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            current_id,
        })
    }

    /// Determines the next available ID by reading existing entries from the file.
    ///
    /// This function reads all existing log entries and returns the next ID
    /// that should be used for new entries.
    ///
    /// # Arguments
    ///
    /// * `file` - The file handle to read from.
    ///
    /// # Returns
    ///
    /// Returns the next available ID (0 if no entries exist).
    fn get_new_id(file: &File) -> Result<u64> {
        let mut file = file.try_clone()?;
        file.seek(SeekFrom::Start(0))?;

        let mut logs = String::new();
        file.read_to_string(&mut logs)?;

        let entries: Vec<LogEntry> = logs
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| serde_json::from_str::<LogEntry>(line).ok())
            .collect();

        if let Some(latest_log) = entries.last() {
            return Ok(latest_log.id + 1);
        }

        Ok(0)
    }

    /// Appends data to the Write-Ahead Log.
    ///
    /// This function creates a new log entry with the provided data, assigns
    /// it a unique ID and timestamp, and writes it to disk. The entry is
    /// immediately flushed to ensure durability.
    ///
    /// # Arguments
    ///
    /// * `data` - The binary data to append to the log.
    ///
    /// # Returns
    ///
    /// Returns the created `LogEntry` or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waly::WriteAheadLog;
    ///
    /// let mut wal = WriteAheadLog::new("logs.wal")?;
    /// let entry = wal.append(b"important data".to_vec())?;
    /// println!("Appended entry: {:?}", entry);
    /// ```
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

    /// Reads all entries from the Write-Ahead Log.
    ///
    /// This function reads the entire log file and deserializes all valid
    /// entries. Invalid or corrupted entries are silently skipped.
    ///
    /// # Returns
    ///
    /// Returns a vector of all `LogEntry` instances or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waly::WriteAheadLog;
    ///
    /// let wal = WriteAheadLog::new("logs.wal")?;
    /// let entries = wal.read_all()?;
    ///
    /// for entry in entries {
    ///     println!("Entry {}: {:?}", entry.id, entry.data);
    /// }
    /// ```
    pub fn read_all(&self) -> Result<Vec<LogEntry>> {
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start(0))?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let entries: Vec<LogEntry> = content
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| serde_json::from_str::<LogEntry>(line).ok())
            .collect();

        Ok(entries)
    }

    /// Clears all entries from the Write-Ahead Log.
    ///
    /// This function truncates the log file to zero length, effectively
    /// removing all entries. The file remains open and can be used for
    /// new entries.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waly::WriteAheadLog;
    ///
    /// let wal = WriteAheadLog::new("logs.wal")?;
    /// wal.clear()?;
    /// println!("Log cleared successfully");
    /// ```
    pub fn clear(&self) -> Result<()> {
        let mut file = self.file.lock().unwrap();
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    /// Removes a specific entry from the Write-Ahead Log by ID.
    ///
    /// This function reads all entries, filters out the entry with the specified ID,
    /// and rewrites the file with the remaining entries. This operation is not
    /// atomic and should be used carefully in production environments.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the entry to remove.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waly::WriteAheadLog;
    ///
    /// let wal = WriteAheadLog::new("logs.wal")?;
    /// wal.clear_id(123)?;
    /// println!("Entry with ID 123 removed");
    /// ```
    pub fn clear_id(&self, id: u64) -> Result<()> {
        println!("clearing id: {:?}", id);
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start(0))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let entries: Vec<u8> = content
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| serde_json::from_str::<LogEntry>(line).ok())
            .filter(|entry| entry.id != id)
            .flat_map(|entry| {
                let mut bytes = serde_json::to_vec(&entry).unwrap();
                bytes.push(b'\n');
                bytes
            })
            .collect();

        file.set_len(0)?;
        file.write_all(&entries)?;
        file.flush()?;
        Ok(())
    }
}
