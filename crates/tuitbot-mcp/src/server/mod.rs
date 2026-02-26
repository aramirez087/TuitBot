//! MCP server implementations.
//!
//! - [`WriteMcpServer`]: standard write profile (all typed tools, no universal requests).
//! - [`AdminMcpServer`]: admin profile (superset of write, adds universal request tools).
//! - [`ReadonlyMcpServer`]: minimal readonly profile (10 tools, no DB).
//! - [`ApiReadonlyMcpServer`]: broader api-readonly profile (20 tools, no DB).

pub mod admin;
pub mod api_readonly;
pub mod readonly;
pub mod write;

pub use admin::AdminMcpServer;
pub use api_readonly::ApiReadonlyMcpServer;
pub use readonly::ReadonlyMcpServer;
pub use write::WriteMcpServer;
