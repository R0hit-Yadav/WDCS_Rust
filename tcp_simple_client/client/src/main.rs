use std::{net::TcpStream, thread::sleep, time::Duration};
use std::io::Write;
use ed25519_dalek::{SigningKey, Signer};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::Serialize;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::StreamExt;

#[derive(Serialize)]
struct ClientData {
    client_id: usize,
    avg_price: f32,
    signature: Vec<u8>,
}

#[tokio::main]
async fn main() {
    let client_id = 1; // Unique client ID
    let mut csprng = OsRng;
    let mut random_bytes = [0u8; 32];
    csprng.fill_bytes(&mut random_bytes);

    let signing_key = SigningKey::from_bytes(&random_bytes); // Correct signing key initialization

    let req = "wss://stream.binance.com:9443/ws/btcusdt@trade";
    let (mut socket, _) = connect_async(req).await.expect("Failed to connect");

    let mut price_vec = vec![];
    for _ in 0..10 {
        if let Some(Ok(Message::Text(text))) = socket.next().await {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(price_str) = parsed["p"].as_str() {
                    if let Ok(price) = price_str.parse::<f32>() {
                        // sleep(Duration::from_secs(1));
                        println!("Received price: {:.5}", price);
                        price_vec.push(price);
                    }
                }
            }
        }
    }

    if !price_vec.is_empty() {
        let avg_price = price_vec.iter().sum::<f32>() / price_vec.len() as f32;

        println!("Calculated average price: {:.2}", avg_price);
        
        // Sign both client_id and avg_price
        let signature = signing_key.sign(format!("{}{:?}", client_id, avg_price).as_bytes());
        println!("Signature: {:?}", signature.to_bytes());

        let client_data = ClientData {
            client_id,
            avg_price,
            signature: signature.to_bytes().to_vec(),
        };

        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {
            let serialized = serde_json::to_vec(&client_data).unwrap();
            stream.write_all(&serialized).expect("Failed to send data");
        } else {
            println!("Failed to connect to server.");
        }
    }
}
