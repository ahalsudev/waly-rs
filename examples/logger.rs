/// This example demonstrates how to use the WriteAheadLog and its methods
/// The assumed scenario is that your software is persisting logs to disk to help recover
/// from a crash or network failures.
use waly_rs::WriteAheadLog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a new WriteAheadLog instance
    let mut wal = WriteAheadLog::new("logs.wal")?;

    // 2. Persist log to disk before processing
    let log_data1 = "Test persistent log 1".as_bytes().to_vec();
    let entry1 = wal.append(log_data1.clone())?;

    // 3. Process log data
    let mut processed = true;
    println!(
        "processing log: {:?}",
        String::from_utf8(log_data1).unwrap()
    );

    // 4. Remove processed logs form the wal file
    if processed {
        wal.clear_id(entry1.id)?;
    }

    // 5. Persist log to disk before processing
    let log_data2 = "Test persistent log 2".as_bytes().to_vec();
    let _entry2 = wal.append(log_data2.clone())?;

    // 6. Set processed to false to simulate a failure
    processed = false;
    println!(
        "processing log: {:?}",
        String::from_utf8(log_data2).unwrap()
    );

    // 7. Somewhere in your code, try to recover all logs
    // 8. Remove the log from the wal file if successfully processed
    if !processed {
        let entries = wal.read_all()?;
        for entry in entries {
            println!("re-transmitting log id: {:?}", entry.id);
            println!("entry: {:?}", entry);
            wal.clear_id(entry.id)?;
            println!("transimission successful");
        }
    }

    Ok(())
}
