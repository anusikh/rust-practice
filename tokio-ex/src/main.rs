use std::{
    str,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let balance = Arc::new(Mutex::new(0.00f32));
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        let balance = balance.clone();
        let (stream, _) = listener.accept().await.unwrap();
        // tokio spawn since want to process event async, not one at a time
        tokio::spawn(async move {
            handle_connection(stream, balance).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream, balance: Arc<Mutex<f32>>) {
    // read 1st 16 characters from incoming stream
    let mut buffer = [0; 16];
    stream.read(&mut buffer).await.unwrap();
    println!("{:?}", buffer);

    // read 1st 4 characters of buffer
    let method_type = match str::from_utf8(&buffer[0..4]) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    println!("{:?}", str::from_utf8(&buffer[0..16]));

    let contents = match method_type {
        "GET " => {
            format!("{{\"balance\": {}}}", balance.lock().unwrap())
        }
        "POST" => {
            // ex: POST /-12.98 HTT This string is received, removing the space below
            // we only consider /-12.98 HTT
            let input: String = buffer[6..16]
                .iter()
                .take_while(|x| **x != 32u8)
                .map(|x| *x as char)
                .collect();
            let balance_update: f32 = input.parse::<f32>().unwrap();

            // we have used an arc mutex in this example because we want concurrency and also we want only one thread to access the float value at a time
            let mut locked_balance: std::sync::MutexGuard<'_, f32> = balance.lock().unwrap();
            *locked_balance += balance_update;
            format!("{{\"balance\": {}}}", locked_balance)
        }
        _ => {
            panic!("invalid http method");
        }
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}
