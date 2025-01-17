use std::io::{Read, Write}; 
use std::net::TcpStream; // TcpStream for network communication
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?; // Connect to the server
    println!("Client 1 Connected to the server!"); // successful connection

    loop {
        let mut message = String::new(); // store the message
        print!("Enter message for server: "); // input
        io::stdout().flush()?; // Flush the standard output to ensure the prompt is displayed

        io::stdin().read_line(&mut message)?; // Read user input from standard input

        if message.trim() == "exit" { // Check if the user wants to exit
            println!("Disconnecting from the server..."); 
            break;
        }

        stream.write_all(message.as_bytes())?; // Send the message to the server

        let mut buffer = [0; 1024]; // Buffer to store the server's response
        let bytes_read = stream.read(&mut buffer)?; // Read the server's response

        if bytes_read > 0 { // check there is a response
            let response = String::from_utf8_lossy(&buffer[..bytes_read]); // Convert bytes to string
            println!("Server: {}", response.trim_end()); 
        }
    }

    Ok(())
}
