mod client;

use std::io::Error;

pub async fn start() -> Result<(), Error> {
    // Retrieve addresse
    let addr = "127.0.0.1:8080".to_string();
    // Bind a socket to addresse
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);
    let arcmutex_socket_inputs: std::sync::Arc<
        tokio::sync::Mutex<Vec<tokio::net::tcp::OwnedWriteHalf>>,
    > = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
    // Loop
    loop {
        println!("Waiting");
        let (socket, address) = listener.accept().await?;
        println!("Accept from {}", address);
        // Retrieve input and output of the socket
        let (output, input) = socket.into_split();
        let mut socket_input = arcmutex_socket_inputs.lock().await;
        socket_input.push(input);
        client::start(output, std::sync::Arc::clone(&arcmutex_socket_inputs));
        // Retrieve input and output of the socket
    }
}
