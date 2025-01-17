use std::io::{Read, Write};
use std::net::TcpStream;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("Client 3 Connected to the server!");

    loop {
        let mut message = String::new();
        print!("Enter message for server: ");
        io::stdout().flush()?;

        io::stdin().read_line(&mut message)?;

        if message.trim() == "exit" {
            println!("Disconnecting from the server...");
            break;
        }

        stream.write_all(message.as_bytes())?;

        let mut buffer = [0; 1024];
        let bytes_read = stream.read(&mut buffer)?;

        if bytes_read > 0 {
            let response = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("Server: {}", response.trim_end());
        }
    }

    Ok(())
}
