use agora;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;

const SIZE: usize = 1024;
const ITERATION: u64 = 976563;
const START: &str = "/start";
const STOP: &str = "/stop";
const DOWNLOADS: u64 = 2;
const UPLOADS: u64 = 8;

#[tokio::test]
async fn download_speedtest() {
    let mut tasks: Vec<
        tokio::task::JoinHandle<Result<(std::time::Duration, u64), std::io::Error>>,
    > = Vec::new();
    // Connect on upload user
    let Ok((_, input_upload)) = agora::services::user::connect().await else {panic!()};
    for _ in 0..DOWNLOADS {
        // Connect on download user
        let Ok((output_download, _)) = agora::services::user::connect().await else {panic!()};
        tasks.push(tokio::spawn(download(output_download)));
    }
    // Start upload task
    let task_upload = tokio::spawn(upload(input_upload));
    // Wait for upload to finish
    let (result_upload, results_download) =
        tokio::join!(task_upload, futures::future::join_all(tasks));
    // Retrieve upload time
    let upload_time = result_upload.unwrap().unwrap();
    // Calculate upload speed
    let upload_speed: f64 = ((SIZE as u64 * ITERATION) / upload_time.as_secs()) as f64;
    println!(
        "Uploaded {} bytes, Upload speed is {} MB/s",
        ITERATION * SIZE as u64,
        upload_speed / 1000000f64
    );
    // Retrieve download times
    let results: Vec<(std::time::Duration, u64)> = results_download
        .into_iter()
        .map(|result_download| result_download.unwrap().unwrap())
        .collect();
    let mut sum_time: f64 = 0f64;
    let mut sum_quantity: u64 = 0;
    // Iterate over each result
    for result in &results {
        let download_time = result.0;
        let quantity = result.1;
        let download_speed: f64 = (quantity / download_time.as_secs()) as f64;
        sum_time += download_speed;
        sum_quantity += quantity;
        println!(
            "Downdload {} bytes, speed is {} MB/s",
            quantity,
            download_speed / 1000000f64
        );
    }
    println!(
        "Download quantity mean: {} bytes | Speed mean: {} MB/s",
        sum_quantity / results.len() as u64,
        sum_time / (1000000f64 * results.len() as f64)
    )
}

#[tokio::test]
async fn upload_speedtest() {
    let mut tasks: Vec<tokio::task::JoinHandle<Result<std::time::Duration, std::io::Error>>> =
        Vec::new();
    for _ in 0..UPLOADS {
        // Connect on download user
        let Ok((_, input_upload)) = agora::services::user::connect().await else {panic!()};
        tasks.push(tokio::spawn(upload(input_upload)));
    }
    // Wait for upload to finish
    let results_upload = futures::future::join_all(tasks).await;
    // Retrieve download times
    let results: Vec<std::time::Duration> = results_upload
        .into_iter()
        .map(|result_upload| result_upload.unwrap().unwrap())
        .collect();
    let mut sum_time: f64 = 0f64;
    for result in &results {
        let download_time = result;
        let download_speed: f64 = (ITERATION * SIZE as u64 / download_time.as_secs()) as f64;
        sum_time += download_speed;
        println!(
            "Downdload {} bytes, speed is {} MB/s",
            ITERATION * SIZE as u64,
            download_speed / 1000000f64
        );
    }
    println!(
        "Speed mean: {} MB/s",
        sum_time / (1000000f64 * results.len() as f64)
    )
}

/// Upload as many bytes as SIZE * ITERATION
///
/// ## Arguments
/// * input : Input to write into the sockets
async fn upload(mut input: OwnedWriteHalf) -> tokio::io::Result<std::time::Duration> {
    // Create buffer to host size message
    let bytes: [u8; SIZE] = [0u8; SIZE];
    // Write start command
    input.write_all(&START.len().to_ne_bytes()).await?;
    input.write_all(START.as_bytes()).await?;
    // Start timer
    let begin = std::time::Instant::now();
    // Iterate
    for _ in 0..ITERATION {
        // Write bytes
        input.write_all(&bytes.len().to_ne_bytes()).await?;
        input.write_all(&bytes).await?;
    }
    // Send stop command
    input.write_all(&STOP.len().to_ne_bytes()).await?;
    input.write_all(STOP.as_bytes()).await?;
    // Retrieve time from timer
    let elapsed_time = begin.elapsed();
    // Send timer result
    Ok(elapsed_time)
}
/// Download from START to STOP command
///
/// ## Arguments
/// * output : Output to read from the socket
async fn download(mut output: OwnedReadHalf) -> tokio::io::Result<(std::time::Duration, u64)> {
    // Run
    let mut run = true;
    // Start timer
    let mut begin: Option<std::time::Instant> = None;
    // Init byte counter
    let mut counter: u64 = 0u64;
    // While run
    while run {
        // Init size to read in bytes
        let mut size_in_bytes: [u8; 8] = [0u8; 8];
        // Read the size of the message
        output.read_exact(&mut size_in_bytes).await?;
        // Convert size to usize
        let size: usize = usize::from_ne_bytes(size_in_bytes);
        // Init data in bytes
        let mut output_in_bytes = vec![0u8; size];
        // Read the output in bytes
        output.read_exact(&mut output_in_bytes).await?;
        // If start command
        if size == START.len()
            && String::from(std::str::from_utf8(&output_in_bytes).unwrap()) == String::from(START)
        {
            begin = Some(std::time::Instant::now());
        }
        // If stop command
        else if size == STOP.len()
            && String::from(std::str::from_utf8(&output_in_bytes).unwrap()) == String::from(STOP)
        {
            // Stop running
            run = false
        } else {
            counter += size as u64
        }
    }
    // Retrieve timer elapsed time
    if let Some(start_time) = begin {
        return Ok((start_time.elapsed(), counter));
    } else {
        return Err(tokio::io::Error::new(
            tokio::io::ErrorKind::Other,
            "Timer not started",
        ));
    }
}
