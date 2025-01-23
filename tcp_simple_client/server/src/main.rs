use std::net::{TcpListener, TcpStream};//listen server and active connetion 
use std::io::Read; // read data from stram
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};//for spwan thread
use ed25519_dalek::{Signature, Verifier, VerifyingKey}; // for verify signatures
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Clone)]
struct ClientData { // client data 
    client_id: usize,
    avg_price: f32,
    signature: Vec<u8>,
    public_key: Vec<u8>,
    name:String,
}

fn handle_client(mut stream: TcpStream,shared_data: Arc<Mutex<Vec<ClientData>>>,aggregator_data: Arc<Mutex<Vec<f32>>>) {
    let mut buffer: [u8; 512] = [0u8; 512];// temporary  torage 

    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                if let Ok(data) = serde_json::from_slice::<ClientData>(&buffer[..size]) {
                    let mut shared_data = shared_data.lock().unwrap();
                    let mut aggregator_data = aggregator_data.lock().unwrap();

                    // Verify signature
                    let public_key_bytes: [u8; 32] = data.public_key.clone().try_into().expect("Invalid public key length");
                    let public_key = VerifyingKey::from_bytes(&public_key_bytes).expect("Failed to parse public key");
                    let signature_bytes: [u8; 64] = data.signature.clone().try_into().expect("Invalid signature length");
                    let signature = Signature::from_bytes(&signature_bytes);
                    let signed_data = format!("{}{:.5}{}", data.client_id, data.avg_price, data.name);

                    // println!("signed_data: {}", signed_data);
                    // println!("signature: {:?}", signature_bytes);
                    // println!("public_key: {:?}", public_key_bytes);
                    if public_key.verify(signed_data.as_bytes(), &signature).is_ok() 
                    {
                        println!("========================================");
                        println!("|| Successfully Connected!!");

                        // Add client data to the shared storage
                        shared_data.push(data.clone());
                        aggregator_data.push(data.avg_price);
                        // println!("{:?}",aggregator_data);
                        
                        // Print details of all clients
                        for client in shared_data.iter() {
                            println!("|| Client ID: {}", client.client_id);
                            println!("|| Name: {}", client.name);
                            println!("|| Verified And AVG IS: {:.5}", client.avg_price);
                            println!();
                            
                        }
                        // shared_data.clear();

                        // Calculate and print overall average
                        let overall_avg: f32 = aggregator_data.iter().sum::<f32>() / aggregator_data.len() as f32;
                        println!("========================================");
                        println!("|| Overall Average BTC Price: {:.5}", overall_avg);
                        println!("========================================");
                    } else {
                        println!("Failed to verify Client ID: {} Name: {}", data.client_id, data.name);
                    }
                } else {
                    println!("Failed to parse client data");
                }
            }
            Err(e) => {
                println!("Error reading from client: {}", e);
                break;
            }
            _ => break, // Client disconnected
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind server");//listener
    let shared_data = Arc::new(Mutex::new(vec![]));
    let aggregator_data = Arc::new(Mutex::new(vec![]));

    println!("Server is listening on port 8080...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let shared_data = Arc::clone(&shared_data);
                let aggregator_data = Arc::clone(&aggregator_data);//clone data
                thread::spawn(move || {
                    handle_client(stream,shared_data, aggregator_data);
                });
            }
            Err(e) => println!("Connection failed: {}", e),
        }
    }

}
