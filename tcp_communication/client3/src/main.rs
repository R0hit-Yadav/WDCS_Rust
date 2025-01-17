use std::io::{self, Write, Read};
use std::net::TcpStream; // for tcp connetion
use std::thread;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?; // connect to the server
    println!("Client 3 is Connected to the server!");

    let mut read_stream = stream.try_clone()?;
    thread::spawn(move || {
        let mut buffer = [0; 1024]; //storage 
        loop {
            match read_stream.read(&mut buffer) { // read data from the server
                Ok(0) => {
                    println!("Server disconnected");
                    break;
                }
                Ok(_) => {
                    let response = String::from_utf8_lossy(&buffer); // convert the data to string
                    println!("Server: {}", response.trim_end());
                }
                Err(e) => {
                    eprintln!("Failed to read from server: {}", e);
                    break;
                }
            }
        }
    });

    let mut input = String::new(); 
    loop {
        input.clear();
        io::stdin().read_line(&mut input)?; //wait for user input to the server 
        if input.trim().is_empty() { //if empty then terminate loop
            break;
        }
        stream.write_all(input.as_bytes())?;
    }

    Ok(())
}
