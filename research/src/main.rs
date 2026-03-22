use clap::Parser;
use sentinel_research::{run, Args};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let report = run(args).await?;
    println!("{}", report);
    Ok(())
}
