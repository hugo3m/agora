use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Start the client
///
/// ## Arguments
/// * socket_output : socket where to read the output
/// * arcmutex_socket_inputs : Hashmap containing all the sockets input
/// * id : id of the client
pub fn start(
    socket_output: tokio::net::tcp::OwnedReadHalf,
    arcmutex_socket_inputs: std::sync::Arc<
        tokio::sync::Mutex<std::collections::HashMap<u16, tokio::net::tcp::OwnedWriteHalf>>,
    >,
    id: u16,
) {
    // Spawn a new task
    tokio::task::spawn(read_socket_output(
        socket_output,
        Arc::clone(&arcmutex_socket_inputs),
        id,
    ));
}
/// Loop for reading socket output
///
/// ## Arguments
/// * socket_output : socket where to read the output
/// * arcmutex_socket_inputs : Hashmap containing all the sockets input
/// * id : id of the client
async fn read_socket_output(
    mut socket_output: tokio::net::tcp::OwnedReadHalf,
    arcmutex_socket_inputs: std::sync::Arc<
        tokio::sync::Mutex<std::collections::HashMap<u16, tokio::net::tcp::OwnedWriteHalf>>,
    >,
    id: u16,
) -> std::io::Result<()> {
    loop {
        // Init buffer for retrieving size of message
        let mut size_in_bytes: [u8; 8] = [0u8; 8];
        // Read size
        socket_output.read_exact(&mut size_in_bytes).await?;
        // Convert bytes size to usize
        let size: usize = usize::from_ne_bytes(size_in_bytes);
        // Init buffer for reading message
        let mut input_in_bytes = vec![0u8; size];
        // Read nessage
        socket_output.read_exact(&mut input_in_bytes).await?;
        // Write the input into socket inputs
        write_socket_input(
            Arc::clone(&arcmutex_socket_inputs),
            input_in_bytes.to_vec(),
            id,
        )
        .await?;
    }
}
/// Write input into all the socket input
///
/// ## Arguments
/// * arcmutex_socket_inputs : Hashmap containing all the sockets input
/// * input : Input to write into the sockets
/// * id : id of the client
async fn write_socket_input(
    arcmutex_socket_inputs: std::sync::Arc<
        tokio::sync::Mutex<std::collections::HashMap<u16, tokio::net::tcp::OwnedWriteHalf>>,
    >,
    input: Vec<u8>,
    id: u16,
) -> std::io::Result<()> {
    // Acquire HashMap
    let mut socket_inputs = arcmutex_socket_inputs.lock().await;
    // For all entries of the socket inputs
    for (socket_id, socket_input) in socket_inputs.iter_mut() {
        // If the id is different from own id
        if *socket_id != id {
            // Retrieve size of input in bytes
            let size: [u8; 8] = usize::to_ne_bytes(input.len());
            // Write size of the input
            socket_input.write_all(&size).await?;
            // Write input
            socket_input.write_all(&input).await?;
        }
    }
    Ok(())
}
