use std::io::{self};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

pub fn start(
    socket_output: tokio::net::tcp::OwnedReadHalf,
    arcmutex_socket_inputs: Arc<Mutex<Vec<tokio::net::tcp::OwnedWriteHalf>>>,
) {
    tokio::task::spawn(read_socket_output(
        socket_output,
        Arc::clone(&arcmutex_socket_inputs),
    ));
}

async fn read_socket_output(
    mut socket: tokio::net::tcp::OwnedReadHalf,
    arcmutex_socket_inputs: Arc<Mutex<Vec<tokio::net::tcp::OwnedWriteHalf>>>,
) -> io::Result<()> {
    loop {
        let mut size_in_bytes: [u8; 8] = [0u8; 8];
        socket.read_exact(&mut size_in_bytes).await?;
        let size: usize = usize::from_ne_bytes(size_in_bytes);
        let mut input_in_bytes = vec![0u8; size];
        socket.read_exact(&mut input_in_bytes).await?;
        let input = String::from_utf8_lossy(&input_in_bytes);
        write_socket_input(
            Arc::clone(&arcmutex_socket_inputs),
            input_in_bytes.to_vec(),
            size_in_bytes,
        )
        .await?;
        println!("{}", input);
    }
}

async fn write_socket_input(
    arcmutex_socket_inputs: Arc<Mutex<Vec<tokio::net::tcp::OwnedWriteHalf>>>,
    input_in_bytes: Vec<u8>,
    size_in_bytes: [u8; 8],
) -> io::Result<()> {
    let mut socket_inputs = arcmutex_socket_inputs.lock().await;
    for socket_input in socket_inputs.iter_mut() {
        socket_input.write_all(&size_in_bytes).await?;
        socket_input.write_all(&input_in_bytes).await?;
    }
    Ok(())
}
