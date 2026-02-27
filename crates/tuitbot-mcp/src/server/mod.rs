//! MCP server implementations.
//!
//! - [`WriteMcpServer`]: standard write profile (all typed tools, no universal requests).
//! - [`AdminMcpServer`]: admin profile (superset of write, adds universal request tools).
//! - [`ReadonlyMcpServer`]: minimal readonly profile (10 tools, no DB).
//! - [`ApiReadonlyMcpServer`]: broader api-readonly profile (20 tools, no DB).
//! - [`UtilityReadonlyMcpServer`]: flat toolkit reads + scoring + config (no workflow).
//! - [`UtilityWriteMcpServer`]: flat toolkit reads + writes + engages (no workflow).

pub mod admin;
pub mod api_readonly;
pub mod readonly;
mod toolkit_response;
pub mod utility_readonly;
pub mod utility_write;
pub mod write;

pub use admin::AdminMcpServer;
pub use api_readonly::ApiReadonlyMcpServer;
pub use readonly::ReadonlyMcpServer;
pub use utility_readonly::UtilityReadonlyMcpServer;
pub use utility_write::UtilityWriteMcpServer;
pub use write::WriteMcpServer;
