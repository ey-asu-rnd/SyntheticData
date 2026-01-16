//! REST and WebSocket API implementation.

mod routes;
mod websocket;

pub use routes::{create_router, create_router_with_cors, CorsConfig};
pub use websocket::MetricsStream;
