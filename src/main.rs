mod cli;
mod executor;
mod handler;
mod tool_def;

use anyhow::Result;
use clap::Parser;
use rmcp::{transport::io::stdio, ServiceExt};

use cli::Cli;
use handler::ShellHandler;
use tool_def::ToolDef;

#[tokio::main]
async fn main() -> Result<()> {
    // Logs must go to stderr — stdout is reserved for the JSON-RPC MCP transport.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let cli = Cli::parse();
    cli.validate()?;

    let tool_defs: Vec<ToolDef> = cli.tools.iter()
        .zip(cli.descriptions.iter())
        .zip(cli.commands.iter())
        .map(|((name, desc), cmd)| ToolDef::new(name.clone(), desc.clone(), cmd.clone()))
        .collect();

    let service = ShellHandler::new(tool_defs)
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("server error: {e}"))?;

    service.waiting().await?;

    Ok(())
}
