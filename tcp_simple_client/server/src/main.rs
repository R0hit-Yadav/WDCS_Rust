use std::net::{TcpListener, TcpStream};//listen server and active connetion 
use std::io::Read; // read data from stram
use std::sync::{Arc, Mutex};
use std::thread;//for spwan thread
use ed25519_dalek::{Signature, Verifier, VerifyingKey}; // for verify signatures
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ClientData { // client data 
    client_id: usize,
    avg_price: f32,
    signature: Vec<u8>,
    public_key: Vec<u8>,
}

fn handle_client(mut stream: TcpStream,aggregator_data: Arc<Mutex<Vec<f32>>>) {
    let mut buffer: [u8; 512] = [0u8; 512];// temporary  torage 

    match stream.read(&mut buffer) { // read coming data
        Ok(size) => {
            if let Ok(data) = serde_json::from_slice::<ClientData>(&buffer[..size]) { //decode recived JSON data into client data
                let public_key_bytes: [u8; 32] = data.public_key.try_into().expect("Invalid public key length");
                let public_key = VerifyingKey::from_bytes(&public_key_bytes).expect("Failed to parse public key");
                //convert keys into fixed sized array
                let signature_bytes: [u8; 64] = data.signature.try_into().expect("Invalid signature length");
                let signature = Signature::from_bytes(&signature_bytes);

                // println!("Signature Key :{:?}",signature_bytes); // to see keys
                // println!("Public Key :{:?}",public_key);

                let signed_data = format!("{}{:?}", data.client_id, data.avg_price); //original message of client

                if public_key.verify(signed_data.as_bytes(), &signature).is_ok() //verify 
                {     
                    println!("Verified client {} with avg price: {:.5}", data.client_id, data.avg_price);
                    let mut aggregator_data = aggregator_data.lock().unwrap();
                    aggregator_data.push(data.avg_price);//pused to aggregator
                } 
                else 
                {
                    println!("Failed to verify client {}", data.client_id);
                }
            } 
            else 
            {
                println!("Failed to parse client data");
            }
        }
        Err(e) => println!("Failed to read from stream: {}", e),
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind server");//listener
    let aggregator_data = Arc::new(Mutex::new(vec![]));

    println!("Server is listening on port 8080...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let aggregator_data = Arc::clone(&aggregator_data);//clone data
                thread::spawn(move || {
                    handle_client(stream, aggregator_data);
                });
            }
            Err(e) => println!("Connection failed: {}", e),
        }
    }

    let aggregator_data = aggregator_data.lock().unwrap(); //aggregate final avg and print
    if !aggregator_data.is_empty() 
    {
        let overall_avg: f32 = aggregator_data.iter().sum::<f32>() / aggregator_data.len() as f32;
        println!("Overall average BTC price: {:.2}", overall_avg);
    } 
    else 
    {
        println!("No valid data received.");
    }
}
