use std::{net::TcpStream, io::Write};
use ed25519_dalek::{SigningKey, Signer};
use rand::rngs::OsRng;//for random bytess
use rand::RngCore;
use serde::Serialize;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};//for connetion of websockets
use futures_util::StreamExt;

#[derive(Serialize)]
struct ClientData { //client data
    client_id: usize,
    avg_price: f32,
    signature: Vec<u8>,
    public_key: Vec<u8>,
    name:String,
}

#[tokio::main]
async fn main() {
    let client_id = 2; // Unique client ID
    let name = String::from("Patel Dev B.");
    let mut csprng = OsRng;
    let mut random_bytes = [0u8; 32];
    csprng.fill_bytes(&mut random_bytes);

    let signing_key = SigningKey::from_bytes(&random_bytes);//key

    let req = "wss://stream.binance.com:9443/ws/btcusdt@trade";//url for get BTC price 
    let (mut socket, _) = connect_async(req).await.expect("Failed to connect");//socket

    let mut price_vec = vec![];
    for _ in 0..5 {//number of price you have to get 
        if let Some(Ok(Message::Text(text))) = socket.next().await {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(price_str) = parsed["p"].as_str() {
                    if let Ok(price) = price_str.parse::<f32>() {
                        println!("Received price: {:.5}", price);
                        price_vec.push(price);
                    }
                }
            }
        }
    }

    if !price_vec.is_empty() {
        let avg_price = price_vec.iter().sum::<f32>() / price_vec.len() as f32;//calculate avg

        println!("Calculated average price: {:.5}", avg_price);

        let signed_data = format!("{}{:?}{}", client_id, avg_price, name);
        let signature = signing_key.sign(signed_data.as_bytes());
        // println!("Signature: {:?}", signature.to_bytes());

        let public_key = signing_key.verifying_key().to_bytes();
        // println!("Public Key :{:?}",public_key);

        let client_data = ClientData {
            client_id,
            avg_price,
            signature: signature.to_bytes().to_vec(),
            public_key: public_key.to_vec(),
            name,
        };

        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080") {//send data to server
            let serialized = serde_json::to_vec(&client_data).unwrap();
            stream.write_all(&serialized).expect("Failed to send data");
        } else {
            println!("Failed to connect to server.");
        }
    }
}
