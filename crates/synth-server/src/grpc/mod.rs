//! gRPC service implementation for synthetic data generation.

pub mod service;

// Include the generated protobuf code
#[allow(clippy::all)]
#[allow(warnings)]
pub mod synth {
    include!("synth.synth.rs");
}

pub use service::SynthService;
pub use synth::synthetic_data_service_server::SyntheticDataServiceServer;
