use cadentis::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

#[cadentis::test]
async fn file_read_write_roundtrip() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();

    let path = std::env::temp_dir().join(format!(
        "reactor-file-{}-{}.tmp",
        std::process::id(),
        unique
    ));
    let path_string = path.to_string_lossy().into_owned();

    let writer = File::create(&path_string).await.unwrap();
    writer.write_all(b"hello world").await.unwrap();
    drop(writer);

    let reader = File::open(&path_string).await.unwrap();
    let mut buffer = [0u8; 11];
    let n = reader.read(&mut buffer).await.unwrap();

    assert_eq!(n, 11);
    assert_eq!(&buffer[..n], b"hello world");

    let _ = std::fs::remove_file(path);
}
