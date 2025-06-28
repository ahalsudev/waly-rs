# waly_rs

A Stright-forward Write-Ahead Log (WAL) implementation in Rust.

## Features

- **Thread-safe**: Uses `Arc<Mutex<File>>` for concurrent access
- **Durable**: Entries are immediately flushed to disk
- **Simple ID management**: Unique IDs are automatically assigned to entries
- **Flexible data storage**: Supports arbitrary binary data

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
waly_rs = "0.1.4"
```

## Quick Start

```rust
use waly_rs::{WriteAheadLog, Result};

fn main() -> Result<()> {
    // Create a new WAL
    let mut wal = WriteAheadLog::new("logs.wal")?;
    
    // Append some data
    let entry = wal.append(b"Test persistent log".to_vec())?;
    println!("Appended entry with ID: {}", entry.id);
    
    // Read all entries
    let entries = wal.read_all()?;
    for entry in entries {
        println!("Entry {}: {:?}", entry.id, entry.data);
    }
    
    Ok(())
}
```

## API Reference

### WriteAheadLog

The main struct for interacting with the Write-Ahead Log.

#### Methods

##### `new<P: AsRef<Path>>(path: P) -> Result<Self>`

Creates a new Write-Ahead Log at the specified path. If the file doesn't exist, it will be created. If it exists, the log will be opened and the next ID will be determined from existing entries.

```rust
let wal = WriteAheadLog::new("logs.wal")?;
```

##### `append(&mut self, data: Vec<u8>) -> Result<LogEntry>`

Appends data to the Write-Ahead Log. Creates a new log entry with the provided data, assigns it a unique ID and timestamp, and writes it to disk.

```rust
let mut wal = WriteAheadLog::new("logs.wal")?;
let entry = wal.append(b"Test persistent log".to_vec())?;
println!("Appended entry: {:?}", entry);
```

##### `read_all(&self) -> Result<Vec<LogEntry>>`

Reads all entries from the Write-Ahead Log. Invalid or corrupted entries are silently skipped.

```rust
let wal = WriteAheadLog::new("logs.wal")?;
let entries = wal.read_all()?;

for entry in entries {
    println!("Entry {}: {:?}", entry.id, entry.data);
}
```

##### `clear(&self) -> Result<()>`

Clears all entries from the Write-Ahead Log by truncating the file to zero length.

```rust
let wal = WriteAheadLog::new("logs.wal")?;
wal.clear()?;
println!("Log cleared successfully");
```

##### `clear_id(&self, id: u64) -> Result<()>`

Removes a specific entry from the Write-Ahead Log by ID. This operation is not atomic and should be used carefully in production environments.

```rust
let wal = WriteAheadLog::new("logs.wal")?;
wal.clear_id(100)?;
println!("Entry with ID 100 removed");
```

### LogEntry

Represents a single entry in the Write-Ahead Log.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub id: u64,           // Unique identifier
    pub timestamp: u64,    // Unix timestamp
    pub data: Vec<u8>,     // Binary data
}
```

### Error Types

The library uses a custom error type `WalError` that can represent:

- **Io**: I/O errors from file operations
- **Serialization**: JSON serialization/deserialization errors
- **InvalidEntry**: Invalid or corrupted log entries

## Examples

### Basic Usage

```rust
use waly_rs::{WriteAheadLog, Result};

fn main() -> Result<()> {
    let mut wal = WriteAheadLog::new("logs.wal")?;
    
    // Write some data
    wal.append(b"Test persistent log 1".to_vec())?;
    wal.append(b"Test persistent log 2".to_vec())?;
    wal.append(b"Test persistent log 3".to_vec())?;
    
    // Read and display all entries
    let entries = wal.read_all()?;
    for entry in entries {
        let data_str = String::from_utf8_lossy(&entry.data);
        println!("[{}] Entry {}: {}", entry.timestamp, entry.id, data_str);
    }
    
    Ok(())
}
```

## File Format

The WAL file stores entries as JSON objects, one per line:

```
{"id":0,"timestamp":1640995200,"data":[72,101,108,108,111]}
{"id":1,"timestamp":1640995201,"data":[87,111,114,108,100]}
```

Each entry contains:
- `id`: Unique identifier (u64)
- `timestamp`: Unix timestamp when created (u64)
- `data`: Binary data as array of bytes

## Thread Safety

The `WriteAheadLog` is thread-safe and can be shared between multiple threads using `Arc`:

```rust
use std::sync::Arc;
use std::thread;

let wal = Arc::new(WriteAheadLog::new("shared.wal")?);

let wal_clone = wal.clone();
thread::spawn(move || {
    let mut wal = wal_clone;
    wal.append(b"Thread 1 - Test persistent log".to_vec()).unwrap();
});

let wal_clone = wal.clone();
thread::spawn(move || {
    let mut wal = wal_clone;
    wal.append(b"Thread 2 - Test persistent log".to_vec()).unwrap();
});
```

## Performance Considerations

- **File I/O**: Each append operation performs a disk write and flush
- **Memory usage**: `read_all()` loads the entire file into memory
- **Concurrency**: Multiple threads can safely append simultaneously
- **File size**: Consider log rotation for long-running applications

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Changelog

### 0.1.4
- Initial release
- Thread-safe Write-Ahead Log implementation
- JSON-based entry serialization
- Basic recovery and clearance operations
