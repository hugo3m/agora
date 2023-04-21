use std::io::Error;

use tokio::io::AsyncReadExt;

pub async fn start() -> Result<(), Error> {
    let addr = "127.0.0.1:8080".to_string();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    println!("Waiting");
    let (socket, address) = listener.accept().await?;
    println!("Accept from {}", address);
    let task_socket_input = tokio::task::spawn(socket_input(socket));
    let _result = task_socket_input.await?;
    Ok(())
}
/// ### Loop reading the input of the socket and print to stdout
///
/// #### Arguments
///
/// * `socket`: The socket to read from
async fn socket_input(mut socket: tokio::net::TcpStream) -> std::io::Result<()> {
    loop {
        // Init size to read in bytes
        let mut size_in_bytes: [u8; 8] = [0u8; 8];
        // Read the size of the message
        socket.read_exact(&mut size_in_bytes).await?;
        // Convert size to usize
        let size: usize = usize::from_ne_bytes(size_in_bytes);
        // Init data in bytes
        let mut input_in_bytes = vec![0u8; size];
        // Read the input in bytes
        socket.read_exact(&mut input_in_bytes).await?;
        // Convert input to string
        let input = String::from(std::str::from_utf8(&input_in_bytes).unwrap());
        println!("{}", input);
    }
}
