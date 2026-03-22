use clap::Parser;
use sentinel_publisher::{run, Args};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Err(e) = run(args).await {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    }
}
