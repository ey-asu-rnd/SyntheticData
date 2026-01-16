//! REST and WebSocket API implementation.

mod auth;
mod rate_limit;
mod routes;
mod websocket;

pub use auth::{auth_middleware, AuthConfig};
pub use rate_limit::{rate_limit_middleware, RateLimitConfig, RateLimiter};
pub use routes::{create_router, create_router_with_cors, create_router_with_auth, create_router_full, CorsConfig, TimeoutConfig};
pub use websocket::MetricsStream;
