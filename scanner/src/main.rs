use clap::Parser;
use sentinel_media_scanner::{run, Args};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let output = run(args)?;
    let json_output = serde_json::to_string_pretty(&output)?;
    println!("{}", json_output);
    Ok(())
}
