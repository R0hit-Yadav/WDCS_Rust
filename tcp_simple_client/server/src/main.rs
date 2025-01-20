use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::sync::Mutex;
use std::thread;
use ed25519_dalek::{Verifier, VerifyingKey, Signature};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct ClientData {
    client_id: usize,
    avg_price: f32,
    signature: Vec<u8>,
}

fn handle_client(
    stream: TcpStream,
    public_keys: Arc<Vec<VerifyingKey>>,
    aggregator_data: Arc<Mutex<Vec<f32>>>,
) {
    let mut buffer = [0u8; 512];
    let mut stream = stream;

    match stream.read(&mut buffer) {
        Ok(size) => {
            if let Ok(data) = serde_json::from_slice::<ClientData>(&buffer[..size]) {
                let signature = Signature::from_bytes(&data.signature.clone().try_into().expect("Expected 64 bytes"));
                let public_key = &public_keys[data.client_id];
                println!("client connected");
                println!("client id: {}", data.client_id);
                println!("avg price: {}", data.avg_price);
                println!("signature: {:?}", data.signature);
                println!("public key: {:?}", public_key);

                if public_key.verify(&data.avg_price.to_be_bytes(), &signature).is_ok() {
                    println!(
                        "Verified client {} with avg price: {:.2}",
                        data.client_id, data.avg_price
                    );

                    let mut aggregator_data = aggregator_data.lock().unwrap();
                    aggregator_data.push(data.avg_price);
                } else {
                    println!("Failed to verify client {}", data.client_id);
                }
            } else {
                println!("Failed to parse client data");
            }
        }
        Err(e) => println!("Failed to read from stream: {}", e),
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind server");
    let public_keys: Vec<VerifyingKey> = (0..5) // Assume 5 public keys
        .map(|_| VerifyingKey::from_bytes(&[0u8; 32]).unwrap())
        .collect();
    let public_keys = Arc::new(public_keys);
    let aggregator_data = Arc::new(Mutex::new(vec![]));

    println!("Server is listening on port 8080...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let public_keys = Arc::clone(&public_keys);
                let aggregator_data = Arc::clone(&aggregator_data);

                thread::spawn(move || {
                    handle_client(stream, public_keys, aggregator_data);
                });
            }
            Err(e) => println!("Connection failed: {}", e),
        }
    }

    let aggregator_data = aggregator_data.lock().unwrap();
    if !aggregator_data.is_empty() {
        let overall_avg: f32 = aggregator_data.iter().sum::<f32>() / aggregator_data.len() as f32;
        println!("Overall average BTC price: {:.2}", overall_avg);
    } else {
        println!("No valid data received.");
    }
}