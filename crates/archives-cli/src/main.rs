//! Archives CLI
//!
//! Command-line interface for querying logs and metrics.

mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "archives")]
#[command(about = "CLI tool for Archives log and metrics platform")]
#[command(version)]
struct Cli {
    /// API server URL
    #[arg(
        long,
        env = "ARCHIVES_API_URL",
        default_value = "http://localhost:8080"
    )]
    api_url: String,

    /// Output format
    #[arg(long, short, default_value = "table")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Compact,
}

#[derive(Subcommand)]
enum Commands {
    /// Search and view logs
    Logs {
        #[command(subcommand)]
        command: LogsCommands,
    },

    /// Query metrics
    Metrics {
        #[command(subcommand)]
        command: MetricsCommands,
    },

    /// Show system status
    Status,
}

#[derive(Subcommand)]
enum LogsCommands {
    /// Search logs
    Search {
        /// Text to search for
        query: Option<String>,

        /// Time range in hours (default: 1)
        #[arg(long, short = 't', default_value = "1")]
        hours: u32,

        /// Minimum severity level
        #[arg(long, short = 's')]
        severity: Option<String>,

        /// Filter by service name
        #[arg(long)]
        service: Option<String>,

        /// Maximum results
        #[arg(long, short = 'n', default_value = "50")]
        limit: u64,
    },

    /// Tail recent logs
    Tail {
        /// Number of logs to show
        #[arg(short = 'n', default_value = "20")]
        count: u64,

        /// Minimum severity level
        #[arg(long, short = 's')]
        severity: Option<String>,

        /// Filter by service name
        #[arg(long)]
        service: Option<String>,
    },

    /// Show error summary
    Errors {
        /// Time range in hours (default: 24)
        #[arg(long, short = 't', default_value = "24")]
        hours: u32,

        /// Number of top error patterns
        #[arg(long, short = 'n', default_value = "10")]
        limit: u64,
    },
}

#[derive(Subcommand)]
enum MetricsCommands {
    /// List available metrics
    List,

    /// Query a metric
    Query {
        /// Metric name
        name: String,

        /// Time range in hours (default: 1)
        #[arg(long, short = 't', default_value = "1")]
        hours: u32,

        /// Aggregation function
        #[arg(long, short = 'a', default_value = "avg")]
        aggregation: String,

        /// Interval in seconds
        #[arg(long, short = 'i', default_value = "60")]
        interval: u32,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Logs { command } => {
            commands::logs::handle(&cli.api_url, command, cli.format).await?;
        }
        Commands::Metrics { command } => {
            commands::metrics::handle(&cli.api_url, command, cli.format).await?;
        }
        Commands::Status => {
            commands::status::handle(&cli.api_url, cli.format).await?;
        }
    }

    Ok(())
}
