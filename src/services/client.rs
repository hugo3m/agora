use tokio::io::AsyncWriteExt;

use tokio::io::AsyncReadExt;

async fn read_stdin() -> std::io::Result<String> {
    let mut input: String = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("failed to read line");
    Ok(input)
}

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

async fn user_input(mut socket: tokio::net::TcpStream) -> std::io::Result<()> {
    loop {
        let input = read_stdin().await?;
        let size_in_byte: [u8; 8] = input.len().to_ne_bytes();
        socket.write_all(&size_in_byte).await?;
        socket.write_all(input.as_bytes()).await?;
    }
}

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let socket = tokio::net::TcpStream::connect("127.0.0.1:8080").await?;
    println!("TCP Stream connected");
    let task_user_input = tokio::task::spawn(user_input(socket));
    let task_socket_input = tokio::task::spawn(socket_input(socket));
    let _result = task_user_input.await?;
    Ok(())
}
