use waly_rs::WriteAheadLog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wal = WriteAheadLog::new("logs.wal", 1024 * 1024)?;

    let log_data = "Test persistent log".as_bytes().to_vec();
    let _entry = wal.append(log_data)?;

    // Read all entries
    let entries = wal.read_all()?;

    // Process entries that haven't been sent yet
    for entry in entries {
        println!("entry: {:?}", entry);
    }

    Ok(())
}
