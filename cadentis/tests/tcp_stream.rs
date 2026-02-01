#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    #[test]
    fn test_tcp_stream_read_write() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
        let addr = listener.local_addr().expect("Failed to get local address");

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("Failed to accept connection");
            let mut buffer = [0; 5];
            stream
                .read_exact(&mut buffer)
                .expect("Failed to read from stream");
            assert_eq!(&buffer, b"hello");
            stream
                .write_all(b"world")
                .expect("Failed to write to stream");
        });

        let mut stream = TcpStream::connect(addr).expect("Failed to connect to listener");
        stream
            .write_all(b"hello")
            .expect("Failed to write to stream");

        let mut buffer = [0; 5];
        stream
            .read_exact(&mut buffer)
            .expect("Failed to read from stream");
        assert_eq!(&buffer, b"world");

        handle.join().expect("Thread panicked");
    }

    #[test]
    fn test_tcp_stream_multiple_messages() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
        let addr = listener.local_addr().expect("Failed to get local address");

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("Failed to accept connection");
            for _ in 0..3 {
                let mut buffer = [0; 4];
                stream
                    .read_exact(&mut buffer)
                    .expect("Failed to read from stream");
                assert_eq!(&buffer, b"ping");
                stream
                    .write_all(b"pong")
                    .expect("Failed to write to stream");
            }
        });

        let mut stream = TcpStream::connect(addr).expect("Failed to connect to listener");
        for _ in 0..3 {
            stream
                .write_all(b"ping")
                .expect("Failed to write to stream");
            let mut buffer = [0; 4];
            stream
                .read_exact(&mut buffer)
                .expect("Failed to read from stream");
            assert_eq!(&buffer, b"pong");
        }

        handle.join().expect("Thread panicked");
    }
}
