mod client;

use std::io::Error;

/// ### Start the server
pub async fn start() -> Result<(), Error> {
    // Retrieve addresse
    let addr = "127.0.0.1:8080".to_string();
    // Bind a socket to addresse
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);
    // Create an arc mutex HashMap to share between tasks
    let arcmutex_socket_inputs: std::sync::Arc<
        tokio::sync::Mutex<std::collections::HashMap<u16, tokio::net::tcp::OwnedWriteHalf>>,
    > = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
    // Init the first id for connection
    let mut id = 0u16;
    // Loop
    loop {
        println!("Waiting");
        // Wait for new connection
        let (socket, address) = listener.accept().await?;
        println!("Accept from {}", address);
        // Retrieve input and output of the socket
        let (output, input) = socket.into_split();
        // Acquire HashMap
        let mut socket_input = arcmutex_socket_inputs.lock().await;
        // Insert socket into HashMap
        socket_input.insert(id, input);
        // Start the client
        client::start(output, std::sync::Arc::clone(&arcmutex_socket_inputs), id);
        // Increase id
        id += 1;
    }
}
