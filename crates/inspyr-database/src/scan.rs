//! Scan directory and update database. Stub for initial/on-demand scan.

use rusqlite::Result;
use std::path::Path;

/// Scan a directory and sync media into the database.
/// Used for initial or on-demand full scan.
pub fn scan_directory(_path: &Path) -> Result<()> {
    // TODO: walk directory, insert/update rows
    Ok(())
}
