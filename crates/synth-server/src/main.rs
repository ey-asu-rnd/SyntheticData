//! Synthetic Data gRPC Server
//!
//! Starts a gRPC server for synthetic data generation.

use std::net::SocketAddr;

use clap::Parser;
use tonic::transport::Server;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use synth_server::grpc::service::default_generator_config;
use synth_server::{SynthService, SyntheticDataServiceServer};

#[derive(Parser, Debug)]
#[command(name = "synth-server")]
#[command(about = "Synthetic Data gRPC Server", long_about = None)]
struct Args {
    /// Host address to bind to
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    /// Port to listen on
    #[arg(short, long, default_value = "50051")]
    port: u16,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing
    let log_level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Parse address
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;

    info!("Starting Synthetic Data gRPC Server on {}", addr);

    // Create service with default config
    let service = SynthService::new(default_generator_config());

    // Start server
    Server::builder()
        .add_service(SyntheticDataServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
