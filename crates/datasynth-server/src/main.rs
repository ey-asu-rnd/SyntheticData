//! Synthetic Data gRPC Server
//!
//! Starts a gRPC server for synthetic data generation.

use std::net::SocketAddr;
use std::panic;

use clap::Parser;
use tokio::signal;
use tonic::transport::Server;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use datasynth_server::grpc::service::default_generator_config;
use datasynth_server::{SynthService, SyntheticDataServiceServer};

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

    /// Number of worker threads (0 = automatic based on CPU cores)
    #[arg(short, long, default_value = "0")]
    worker_threads: usize,
}

/// Setup panic hook to log panics before aborting.
fn setup_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        error!("Server panic: {}", panic_info);
        default_hook(panic_info);
    }));
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM).
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Setup panic hook for crash logging
    setup_panic_hook();

    // Build tokio runtime with configured worker threads
    let mut runtime_builder = tokio::runtime::Builder::new_multi_thread();
    runtime_builder.enable_all();

    if args.worker_threads > 0 {
        runtime_builder.worker_threads(args.worker_threads);
        eprintln!("Using {} worker threads", args.worker_threads);
    } else {
        // 0 means automatic - tokio defaults to num_cpus
        let num_cpus = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        eprintln!("Using {} worker threads (auto-detected)", num_cpus);
    }

    let runtime = runtime_builder.build()?;

    runtime.block_on(async {
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

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber");

        // Parse address
        let addr: SocketAddr = format!("{}:{}", args.host, args.port)
            .parse()
            .expect("Invalid address");

        info!("Starting Synthetic Data gRPC Server on {}", addr);

        // Create service with default config
        let service = SynthService::new(default_generator_config());

        // Start server with graceful shutdown
        Server::builder()
            .add_service(SyntheticDataServiceServer::new(service))
            .serve_with_shutdown(addr, shutdown_signal())
            .await
            .expect("Server failed");

        info!("Server shutdown complete");
    });

    Ok(())
}
