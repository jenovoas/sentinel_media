use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about = "Sentinel System Ops (Rust Native)")]
struct Args {
    #[arg(short, long)]
    prompt: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage Firewall (firewall-cmd)
    Firewall {
        #[arg(long)]
        status: bool,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        open: Option<String>,
        #[arg(long)]
        close: Option<String>,
    },
    /// Inspect System Logs (journalctl)
    Logs {
        #[arg(long)]
        service: Option<String>,
        #[arg(long)]
        search: Option<String>,
        #[arg(long, default_value_t = 20)]
        lines: usize,
    },
    /// Placeholder for Natural Language Prompts (Currently unhandled in Sysadmin)
    #[command(external_subcommand)]
    External(Vec<String>),
}

fn manage_firewall(
    status_flag: bool,
    list_flag: bool,
    open: Option<String>,
    close: Option<String>,
) -> Result<()> {
    if status_flag {
        let output = Command::new("sudo")
            .args(&["firewall-cmd", "--state"])
            .output()?;
        let state = String::from_utf8_lossy(&output.stdout);
        println!("🔥 Firewall State: {}", state.trim().green().bold());
        return Ok(());
    }

    if list_flag {
        let output = Command::new("sudo")
            .args(&["firewall-cmd", "--list-all"])
            .output()?;
        println!("{}", String::from_utf8_lossy(&output.stdout));
        return Ok(());
    }

    if let Some(port_raw) = open {
        let port_spec = if port_raw.contains('/') {
            port_raw.clone()
        } else {
            format!("{}/tcp", port_raw)
        };
        println!("🔥 Opening port {}...", port_spec.yellow());

        Command::new("sudo")
            .args(&[
                "firewall-cmd",
                &format!("--add-port={}", port_spec),
                "--permanent",
            ])
            .status()
            .context("Failed to add port")?;

        Command::new("sudo")
            .args(&["firewall-cmd", "--reload"])
            .status()
            .context("Failed to reload firewall")?;

        println!("✅ Port {} OPEN.", port_spec.green());
    }

    if let Some(port_raw) = close {
        let port_spec = if port_raw.contains('/') {
            port_raw.clone()
        } else {
            format!("{}/tcp", port_raw)
        };
        println!("🔒 Closing port {}...", port_spec.yellow());

        Command::new("sudo")
            .args(&[
                "firewall-cmd",
                &format!("--remove-port={}", port_spec),
                "--permanent",
            ])
            .status()
            .context("Failed to remove port")?;

        Command::new("sudo")
            .args(&["firewall-cmd", "--reload"])
            .status()
            .context("Failed to reload firewall")?;

        println!("✅ Port {} CLOSED.", port_spec.red());
    }

    Ok(())
}

fn show_logs(service: Option<String>, search: Option<String>, lines: usize) -> Result<()> {
    let mut cmd = Command::new("journalctl");
    cmd.arg("--no-pager").arg("-n").arg(lines.to_string());

    if let Some(s) = service {
        cmd.arg("-u").arg(s);
    }

    if let Some(g) = search {
        cmd.arg("-g").arg(g);
    }

    println!("{}", format!("📄 Logs (n={})...", lines).blue());
    cmd.status()?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Check for prompt flag (natural language)
    if let Some(p) = args.prompt {
        println!(
            "{}",
            "⚠️  [Sysadmin] Natural language prompts not yet implemented for system ops.".yellow()
        );
        println!("   Your prompt was: \"{}\"", p);
        println!("   Please use explicit subcommands: firewall, logs");
        return Ok(());
    }

    // 2. Check for subcommand
    match args.command {
        Some(Commands::Firewall {
            status,
            list,
            open,
            close,
        }) => {
            manage_firewall(status, list, open, close)?;
        }
        Some(Commands::Logs {
            service,
            search,
            lines,
        }) => {
            show_logs(service, search, lines)?;
        }
        Some(Commands::External(args)) => {
            println!(
                "{}",
                "⚠️  [Sysadmin] Unknown command or arguments.".yellow()
            );
            println!("   Debug: {:?}", args);
        }
        None => {
            // No subcommand and no prompt provided
            use clap::CommandFactory;
            Args::command().print_help()?;
        }
    }

    Ok(())
}
