use futures::future::poll_fn;
use futures::task::Poll;
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadBuf};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::Mutex;

/// ### Start the server
pub async fn start() -> Result<(), Error> {
    // Retrieve addresse
    let addr = "127.0.0.1:8080".to_string();
    // Bind a socket to addresse
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);
    // Create an arc mutex HashMap to share between tasks
    let socket_inputs_arc: Arc<Mutex<HashMap<u16, Arc<Mutex<OwnedWriteHalf>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    // Create an arc mutex HashMap for socket output
    let socket_outputs_arc: Arc<Mutex<HashMap<u16, Arc<Mutex<OwnedReadHalf>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    // Create HashMap for running tasks
    let is_running_tasks_arc: Arc<Mutex<HashMap<u16, bool>>> = Arc::new(Mutex::new(HashMap::new()));
    // Create new task peeking socket outputs
    tokio::task::spawn(peek_outputs(
        socket_outputs_arc.clone(),
        socket_inputs_arc.clone(),
        is_running_tasks_arc.clone(),
    ));
    // Init the first id for connection
    let mut id = 0u16;
    // Loop
    loop {
        // Wait for new connection
        let (socket, address) = listener.accept().await?;
        println!("Accept from {}", address);
        // Retrieve input and output of the socket
        let (output, input) = socket.into_split();
        // Acquire HashMaps
        let mut socket_inputs = socket_inputs_arc.lock().await;
        let mut socket_outputs = socket_outputs_arc.lock().await;
        let mut is_running_tasks = is_running_tasks_arc.lock().await;
        // Insert false into is running task for init
        is_running_tasks.insert(id, false);
        // Insert socket into HashMap
        socket_inputs.insert(id, Arc::new(Mutex::new(input)));
        socket_outputs.insert(id, Arc::new(Mutex::new(output)));
        // Increase id
        id += 1;
    }
}

/// Loop for peeking socket outputs
///
/// ## Arguments
/// * socket_outputs_arc : arc mutex hashmap of sockets mapping id to socket output
/// * socket_inputs_src : arc mutex hashmap of sockets mapping id to socket input
/// * is_running_tasks_arc : arc mutex hashmap mapping id to boolean reading task is running
async fn peek_outputs(
    socket_outputs_arc: Arc<Mutex<HashMap<u16, Arc<Mutex<OwnedReadHalf>>>>>,
    socket_intputs_arc: Arc<Mutex<HashMap<u16, Arc<Mutex<OwnedWriteHalf>>>>>,
    is_running_tasks_arc: Arc<Mutex<HashMap<u16, bool>>>,
) {
    loop {
        // Create buffer of size of usize
        let mut buf: [u8; 8] = [0; 8];
        // Acquire socket outputs
        let mut socket_outputs = socket_outputs_arc.lock().await;
        // For all entries of the socket inputs
        for (socket_id, socket_output_arc) in socket_outputs.iter_mut() {
            // Instanciate new buffer
            let mut buf = ReadBuf::new(&mut buf);
            // Copy id of the socket
            let id = *socket_id;
            // Acquire socket output
            let mut socket_output = socket_output_arc.lock().await;
            // Check if the socket is readable
            let is_readable: bool = poll_fn(|cx| match socket_output.poll_peek(cx, &mut buf) {
                // If poll is ready and size to read superior at usize return true
                Poll::Ready(Ok(n)) if n >= 8 => Poll::Ready(true),
                // Otherwise return false
                Poll::Ready(Ok(_)) => Poll::Ready(false),
                Poll::Pending => Poll::Ready(false),
                Poll::Ready(Err(_)) => Poll::Ready(false),
            })
            .await;
            // Drop the lock on socket output
            drop(socket_output);
            // If the socket is reablde
            if is_readable == true {
                // Retrieve is running task hashmap
                let mut is_running_tasks = is_running_tasks_arc.lock().await;
                // Retrieve corresponding running task from id
                let is_running_task = is_running_tasks.get(&id).unwrap();
                // If no task are running
                if *is_running_task == false {
                    // Drop the lock on is running task of the socket
                    drop(is_running_task);
                    // Retrieve again is running task as mutable
                    let is_running_task_mut = is_running_tasks.get_mut(&id).unwrap();
                    // Change the value to true
                    *is_running_task_mut = true;
                    // Spawn new task to read socket
                    tokio::spawn(run_read_socket_output(
                        socket_output_arc.clone(),
                        socket_intputs_arc.clone(),
                        is_running_tasks_arc.clone(),
                        id,
                    ));
                    // Drop the lock on is running task
                    drop(is_running_task_mut);
                }
                // Drop the lock on is running tasks hashmap
                drop(is_running_tasks);
            }
        }
        // Drop the lock on socket outputs
        drop(socket_outputs);
    }
}

async fn run_read_socket_output(
    socket_output_arc: Arc<Mutex<OwnedReadHalf>>,
    socket_inputs_arc: Arc<Mutex<HashMap<u16, Arc<Mutex<OwnedWriteHalf>>>>>,
    is_running_tasks_arc: Arc<Mutex<HashMap<u16, bool>>>,
    id: u16,
) {
    let _task = read_socket_output(socket_output_arc.clone(), socket_inputs_arc.clone(), id).await;
    let mut is_running_tasks = is_running_tasks_arc.lock().await;
    let is_running_task = is_running_tasks.get_mut(&id).unwrap();
    *is_running_task = false;
}

/// Loop for reading socket output
///
/// ## Arguments
/// * socket_output : socket where to read the output
/// * arcmutex_socket_inputs : Hashmap containing all the sockets input
/// * id : id of the client
async fn read_socket_output(
    socket_output_arc: Arc<Mutex<OwnedReadHalf>>,
    socket_inputs_arc: Arc<Mutex<HashMap<u16, Arc<Mutex<OwnedWriteHalf>>>>>,
    id: u16,
) -> std::io::Result<()> {
    // Init buffer for retrieving size of message
    let mut size_in_bytes: [u8; 8] = [0u8; 8];
    // Get socket output
    let mut socket_output = socket_output_arc.lock().await;
    // Read size
    socket_output.read_exact(&mut size_in_bytes).await?;
    // Convert bytes size to usize
    let size: usize = usize::from_ne_bytes(size_in_bytes);
    // Init buffer for reading message
    let mut input_in_bytes = vec![0u8; size];
    // Read nessage
    socket_output.read_exact(&mut input_in_bytes).await?;
    drop(socket_output);
    // Write the input into socket inputs
    tokio::spawn(write_socket_input(
        socket_inputs_arc.clone(),
        input_in_bytes.to_vec(),
        id,
    ));
    return Ok(());
}

/// Write input into all the socket input
///
/// ## Arguments
/// * arcmutex_socket_inputs : Hashmap containing all the sockets input
/// * input : Input to write into the sockets
/// * id : id of the client
async fn write_socket_input(
    socket_inputs_arc: Arc<Mutex<HashMap<u16, Arc<Mutex<OwnedWriteHalf>>>>>,
    input: Vec<u8>,
    id: u16,
) -> std::io::Result<()> {
    // Acquire HashMap
    let mut socket_inputs = socket_inputs_arc.lock().await;
    // For all entries of the socket inputs
    for (socket_id, socket_input_arc) in socket_inputs.iter_mut() {
        // If the id is different from own id
        if *socket_id != id {
            let mut socket_input = socket_input_arc.lock().await;
            // Retrieve size of input in bytes
            let size: [u8; 8] = usize::to_ne_bytes(input.len());
            // Write size of the input
            socket_input.write_all(&size).await?;
            // Write input
            socket_input.write_all(&input).await?;
        }
    }
    return Ok(());
}
