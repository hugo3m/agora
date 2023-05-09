use agora;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;

#[tokio::test]
async fn hello() {
    let Ok((_, input_upload)) = agora::services::user::connect().await else {panic!()};
    let Ok((output_download, _)) = agora::services::user::connect().await else {panic!()};
    let task_upload = tokio::spawn(upload_task(input_upload));
    let task_download = tokio::spawn(download_task(output_download));
    let (time, output) = tokio::join!(task_upload, task_download);
    let e = output.unwrap().unwrap();
    let d = time.unwrap().unwrap();
    println!("{:?}", d);
    println!("{:?}", e)
}

async fn upload_task(mut input: OwnedWriteHalf) -> tokio::io::Result<std::time::Duration> {
    let start = String::from("start");
    let stop = String::from("stop");
    let bytes: [u8; 1024] = [0u8; 1024];
    let begin = std::time::Instant::now();
    input.write_all(&start.len().to_ne_bytes()).await?;
    input.write_all(start.as_bytes()).await?;
    for _ in 0..976563 {
        input.write_all(&bytes.len().to_ne_bytes()).await?;
        input.write_all(&bytes).await?;
    }
    input.write_all(&stop.len().to_ne_bytes()).await?;
    input.write_all(stop.as_bytes()).await?;
    let elapsed_time = begin.elapsed();
    Ok(elapsed_time)
}

async fn download_task(mut output: OwnedReadHalf) -> tokio::io::Result<std::time::Duration> {
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
    // Convert output to string
    let start = String::from(std::str::from_utf8(&output_in_bytes).unwrap());
    if start == String::from("start") {
        let begin = std::time::Instant::now();
        let mut run = true;
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
            let output_to_string = String::from(std::str::from_utf8(&output_in_bytes).unwrap());
            if output_to_string == String::from("stop") {
                run = false
            }
        }
        let elapsed_time = begin.elapsed();
        Ok(elapsed_time)
    } else {
        panic!()
    }
}
