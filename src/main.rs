mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Fail parsing argument");
    }

    if &args[1][..] == "client" {
        services::client::start().await?;
    }

    if &args[1][..] == "server" {
        services::server::start().await?;
    }

    Ok(())
}
