//! PC Controller MCP Server - CLI Entry Point
//!
//! Multi-protocol MCP server for PC control

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod error;
mod platform;
mod tools;
mod protocol;

use protocol::{http, stdio, ws};

/// Protocol to use for MCP communication
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Protocol {
    /// STDIO transport (for local agent integration like Claude Code)
    Stdio,
    /// HTTP transport with Streamable HTTP
    Http,
    /// WebSocket transport
    Ws,
}

/// CLI arguments
#[derive(clap::Parser, Debug)]
#[command(name = "pc-controller-mcp")]
#[command(version = "0.1.0")]
#[command(about = "MCP server for PC control - screen capture, window management, input simulation")]
struct Args {
    /// Protocol to use
    #[arg(short, long, value_enum, default_value_t = Protocol::Stdio)]
    protocol: Protocol,

    /// Address to bind (for HTTP and WebSocket)
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    addr: String,

    /// Enable CORS (for HTTP)
    #[arg(long)]
    cors: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn init_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"))
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(filter)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_logging(args.verbose);

    let addr: std::net::SocketAddr = args.addr.parse()
        .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

    match args.protocol {
        Protocol::Stdio => {
            tracing::info!("Starting PC Controller MCP server in STDIO mode");
            stdio::run_auto().await?;
        }
        Protocol::Http => {
            tracing::info!("Starting PC Controller MCP server in HTTP mode at {}", addr);
            http::run_auto(addr, args.cors).await?;
        }
        Protocol::Ws => {
            tracing::info!("Starting PC Controller MCP server in WebSocket mode at {}", addr);
            ws::run_auto(addr).await?;
        }
    }

    Ok(())
}
