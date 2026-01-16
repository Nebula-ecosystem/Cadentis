//! Example: Asynchronous file read/write with Cadentis

use cadentis::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

#[cadentis::main]
async fn main() {
    // Generate a unique temporary file path
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();

    let path = std::env::temp_dir().join(format!(
        "cadentis-file-{}-{}.tmp",
        std::process::id(),
        unique
    ));
    let path_string = path.to_string_lossy().into_owned();

    // Write to the file asynchronously
    let writer = File::create(&path_string).await.unwrap();
    writer.write_all(b"hello world").await.unwrap();
    drop(writer);

    // Read from the file asynchronously
    let reader = File::open(&path_string).await.unwrap();
    let mut buffer = [0u8; 11];
    let n = reader.read(&mut buffer).await.unwrap();

    println!("Read {} bytes: {:?}", n, &buffer[..n]);

    // Clean up the temporary file
    let _ = std::fs::remove_file(path);
}
