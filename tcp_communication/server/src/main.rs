use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};// allow the server to listen incoming TCP connections
use std::thread;


fn handle_client(mut stream: TcpStream) { // 
    let mut buffer = [0; 1024]; //temporary storage for incoming data
    loop {
        match stream.read(&mut buffer) { // read incoming data from the client
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(_) => {
                let received = String::from_utf8_lossy(&buffer); //if data received, convert it to a string
                println!("Client: {}", received.trim_end());
                stream.write_all(b"Message received").unwrap();
            }
            Err(e) => {
                eprintln!("Failed to read from client: {}", e);
                break;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;// start listening on ip address and port
    println!("Server listening on port 8080...");

    for stream in listener.incoming() { //a loop to accept incoming connections
        match stream {
            Ok(stream) => {
                println!("New client connected!");
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}
