use rand::rngs::OsRng; // for random number
use rand::RngCore; // to use fill_bytes
use ed25519_dalek::{SigningKey, VerifyingKey, Signature}; // dalek library for signing and verifying
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant}; // for sleep duration time and timer
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message}; // for connect socket
use futures_util::StreamExt; // for StreamExt
use serde_json; // for json
use clap::{Arg, Command}; // for argument parsing
use std::fs;
use ed25519_dalek::{Signer, Verifier};



#[tokio::main]
pub async fn main() { // using clap for argument parsing
    let matches = Command::new("WebSocket Client") 
        .version("1.0") 
        .author("Rohit Yadav <rohitkyadav2312@gmail.com>")
        .about("Task:Simulated distributed client Using signatures")
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_parser(["cache", "read"])
                .required(true)
                .help("Mode of operation: 'cache'=> for  aggregate or 'read'=> for read stored data"),
        )
        .arg(
            Arg::new("times")
                .long("times")
                .value_parser(clap::value_parser!(u64))
                .default_value("5")
                .help("Number of seconds for the WebSocket stream"),
        )
        .get_matches();

    let mode = matches.get_one::<String>("mode").expect("Mode is required");
    let times = matches.get_one::<u64>("times").copied().unwrap_or(5);

    match mode.as_str() {
        "cache" => distributed_client(times).await,
        "read" => read_get_data(),
        _ => println!("Enter Correct Mode"),
    }
}



async fn distributed_client(duration: u64)  // function to  simulate distributed clients
{
    let n_clients = 5;
    let (tx, mut rx) = mpsc::channel::<(usize, f32, Signature)>(n_clients);

    let mut signing_keys = vec![]; // vectors to store keys
    let mut public_keys = vec![];

    for _ in 0..n_clients { 
        let mut csprng = OsRng; // generate  signingKeys for all clients
        let mut random_bytes = [0u8; 32];
        csprng.fill_bytes(&mut random_bytes);
        let signing_key = SigningKey::from_bytes(&random_bytes); // store the signing keys

        public_keys.push(signing_key.verifying_key()); // store the public  keys
        signing_keys.push(signing_key);
    }

    for i in 0..n_clients {
        let tx = tx.clone(); // clone the sender for  every client
        let signing_key = signing_keys[i].clone();

        tokio::spawn(async move {
            client_process(i, duration, tx, signing_key).await; // call the client process
        });
    }

    aggregator(&mut rx, n_clients, &public_keys).await; //  call the aggregator
}

async fn client_process(id: usize,duration: u64,tx: mpsc::Sender<(usize, f32, Signature)>,signing_key: SigningKey) 
{
    let req = "wss://stream.binance.com:9443/ws/btcusdt@trade"; // request to the server
    let (mut socket, _) = connect_async(req).await.expect("Connection failed"); // connect to the socket
    println!("Client no {} is connected", id);

    let mut price_vec = vec![]; //  vector to store the price
    let start_time = Instant::now();


    // chnage in this loop
    while start_time.elapsed() < Duration::from_secs(duration) 
    {
        match socket.next().await 
        {
            Some(Ok(Message::Text(text))) => 
            {
                match serde_json::from_str::<serde_json::Value>(&text) 
                {
                    Ok(parsed) => 
                    {
                        if let Some(price_str) = parsed["p"].as_str() 
                        {
                            match price_str.parse::<f32>() 
                            {
                                Ok(price) => {price_vec.push(price);}
                                Err(e) => println!("Failed to parse price: {}", e),
                            }
                        }
                    }Err(e) => println!("Failed to parse Data: {}", e),
                }
            }
            Some(Ok(_)) => {} // error handling
            Some(Err(e)) => {println!("Error receiving message: {}", e)}
            _ => {break} // Break the loop if the stream ends
        }
    
        //  if the time take grater than the duration then break the loop
        if start_time.elapsed() >= Duration::from_secs(duration) {
            break;
        }
    }

    if !price_vec.is_empty() // check if the vector is not empty
    {
        let avg = price_vec.iter().sum::<f32>() / price_vec.len() as f32; //  calculate the avg price
        println!("Client no {} avg is: {}", id, avg);

        let signature = signing_key.sign(&avg.to_be_bytes()); //  sign the avg price
        tx.send((id, avg, signature)).await.expect("Failed to send");
    } 
    else 
    {
        println!("Client no {} received no valid price data", id); // error handling
    }
}

async fn aggregator(rx: &mut mpsc::Receiver<(usize, f32, Signature)>,n_clients: usize,public_keys: &[VerifyingKey]) 
{
    let mut avg = vec![];

    for _ in 0..n_clients 
    {
        if let Some((client_id, avg_price, signature)) = rx.recv().await 
        {
            if public_keys[client_id]
                .verify(&avg_price.to_be_bytes(), &signature)
                .is_ok()
            {
                println!("Aggregator verified and received avg {} from client no {}",avg_price, client_id);
                avg.push(avg_price);
            } else 
            {
                println!("Aggregator could not verify signature for client no {}",client_id);
            }
        }
    }

    if !avg.is_empty()  //  check if the vector is not empty
    {
        let overall_avg = avg.iter().sum::<f32>() / avg.len() as f32;
        println!("The overall avg BTC price in USD is: {}", overall_avg);

        let data = format!("Client avg: {:?} Overall avg: {}", avg, overall_avg);
        fs::write("aggregated_price_of_btc_in_usd.txt", data).expect("Cannot write to file");
    } else {
        println!("No valid data to aggregate."); //  error handling
    }
}

fn read_get_data() 
{
    match fs::read_to_string("aggregated_price_of_btc_in_usd.txt") 
    {
        Ok(data) => println!("Data of file is: {}", data),
        Err(e) => eprintln!("Cannot read file: {}", e),
    }
}
