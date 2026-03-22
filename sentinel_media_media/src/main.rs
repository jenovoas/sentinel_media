use clap::Parser;
use sentinel_media_media::{run, Args};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args).await?;
    Ok(())
}
