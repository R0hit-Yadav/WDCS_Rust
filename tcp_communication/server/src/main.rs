use std::io::{Read, Write}; 
use std::net::{TcpListener, TcpStream}; // TcpListener and TcpStream for network communication
use std::sync::{Arc, Mutex}; // Arc and Mutex for thread-safe shared state
use std::thread; // thread module for spawning threads

fn handle_client(mut stream: TcpStream, client_id: usize) {
    let mut buffer = [0; 1024]; // Buffer to store incoming data
    println!("Client {} connected.", client_id); 

    loop {
        match stream.read(&mut buffer) { // Read data from the client
            Ok(0) => {
                println!("Client {} disconnected.", client_id);
                break;
            }
            Ok(bytes_read) => {
                let received = String::from_utf8_lossy(&buffer[..bytes_read]); // Convert bytes to string
                println!("Client {}: {}", client_id, received.trim_end());

                let response = format!("Message received from Client {}", client_id); // Create response message
                if let Err(e) = stream.write_all(response.as_bytes()) { // Send response to client
                    eprintln!("Failed to send response to Client {}: {}", client_id, e); // error if sending fails
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read from Client {}: {}", client_id, e); // error if reading fails
                break;
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?; // listener to the address and port
    println!("Server listening on port 8080...");

    let client_counter = Arc::new(Mutex::new(0)); // Shared counter for client IDs

    for stream in listener.incoming() { // Accept incoming connections
        match stream {
            Ok(stream) => {
                let client_id = {
                    let mut counter = client_counter.lock().unwrap();
                    *counter += 1;
                    *counter 
                };

                let client_thread = thread::spawn(move || { // Spawn a new thread for the client
                    handle_client(stream, client_id); // Handle the client in the new thread
                });

                client_thread.join().unwrap_or_else(|_| println!("Client {} thread failed.", client_id)); 
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}
