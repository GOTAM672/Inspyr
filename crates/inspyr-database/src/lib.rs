//! Shared database layer: connection, schema, CRUD, and scan.
//! Used by both the UI (read) and the daemon (write).

mod connection;
mod operation;
mod scan;

pub use connection::Database;
pub use operation::DatabaseOperations;
pub use scan::scan_directory;
