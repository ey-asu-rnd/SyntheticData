//! REST and WebSocket API implementation.

mod routes;
mod websocket;

pub use routes::create_router;
pub use websocket::MetricsStream;
