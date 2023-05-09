use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

/// ### Read the stdin and sends the result in string
async fn read_stdin() -> std::io::Result<String> {
    let mut input: String = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("failed to read line");
    Ok(input)
}
/// ### Loop reading the output of the socket and printing to stdout
///
/// ## Arguments
/// * socket_output : socket where to read the output and write to stdout
pub async fn read_socket_output(
    mut socket_output: tokio::net::tcp::OwnedReadHalf,
) -> std::io::Result<()> {
    // Loop
    loop {
        // Init size to read in bytes
        let mut size_in_bytes: [u8; 8] = [0u8; 8];
        // Read the size of the message
        socket_output.read_exact(&mut size_in_bytes).await?;
        // Convert size to usize
        let size: usize = usize::from_ne_bytes(size_in_bytes);
        // Init data in bytes
        let mut output_in_bytes = vec![0u8; size];
        // Read the output in bytes
        socket_output.read_exact(&mut output_in_bytes).await?;
        // Convert output to string
        let output = String::from(std::str::from_utf8(&output_in_bytes).unwrap());
        // Print output
        println!("{}", output);
    }
}
/// ### Loop reading the output from stdin and writing to socket input
///
/// ## Arguments
/// * socket_input : socket where to write the output of the stdin
pub async fn read_stdin_output(
    socket_input: tokio::net::tcp::OwnedWriteHalf,
) -> std::io::Result<()> {
    let arc_socket_input = Arc::new(Mutex::new(socket_input));
    // Loop
    loop {
        // Read stdin to become socket input
        let input = read_stdin().await?;
        let clone = arc_socket_input.clone();
        let _task = write_socket_input(input, clone).await;
    }
}

pub async fn write_socket_input(
    input: String,
    arc_socket_input: Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>,
) -> tokio::io::Result<()> {
    let mut socket_input = arc_socket_input.lock().await;
    // Write size
    socket_input.write_all(&input.len().to_ne_bytes()).await?;
    // Write stdin output
    socket_input.write_all(input.as_bytes()).await?;
    drop(socket_input);
    // Returns ok behavior
    return Ok(());
}

pub async fn connect() -> Result<(OwnedReadHalf, OwnedWriteHalf), Box<dyn std::error::Error>> {
    let socket = tokio::net::TcpStream::connect("127.0.0.1:8080").await?;
    println!("TCP Stream connected");
    return Ok(socket.into_split());
}

/// ### Start the client who tries to connect and create two task for reading socket and stdin
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let (output, input) = connect().await?;
    // Create task reading output from stdin
    let task_read_user_input = tokio::task::spawn(read_stdin_output(input));
    // Create task reading output from socket
    let task_read_socket_output = tokio::task::spawn(read_socket_output(output));
    // Wait for task to finish
    let _result_read_user_input = task_read_user_input.await?;
    let _result_read_socket_output = task_read_socket_output.await?;
    Ok(())
}
