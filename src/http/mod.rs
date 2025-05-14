mod request;
mod response;
pub mod server;
mod status;

pub use request::{ParsedRequest, Request};
pub use response::Response;
pub use status::Status;

pub const VERSION: &[u8; 8] = b"HTTP/1.1";
pub const EOL: &[u8; 2] = b"\r\n";
pub const EOH: &[u8; 4] = b"\r\n\r\n";
