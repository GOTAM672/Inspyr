//! CRUD operations for the gallery database.

use crate::Database;
use rusqlite::Result;
use std::path::PathBuf;

// -----------------------------------------------------------------------------
// Model (matches tables.sql: images)
// -----------------------------------------------------------------------------

/// One row from the `images` table.
#[derive(Debug, Clone)]
pub struct Image {
    pub id: i64,
    pub path: PathBuf,
    pub filename: String,
}

/// Input for inserting a new image (id is auto-generated).
#[derive(Debug, Clone)]
pub struct InsertImage {
    pub path: PathBuf,
    pub filename: String,
}

/// Optional fields for partial update (only set fields are updated).
#[derive(Debug, Clone, Default)]
pub struct UpdateImage {
    pub path: Option<PathBuf>,
    pub filename: Option<String>,
}

// -----------------------------------------------------------------------------
// Query options (for list/search)
// -----------------------------------------------------------------------------

/// Pagination and optional filters for listing images.
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub limit: u32,
    pub offset: u32,
    // Future: pub album_id: Option<i64>, pub since: Option<DateTime>, etc.
}

// -----------------------------------------------------------------------------
// DatabaseOperations
// -----------------------------------------------------------------------------

/// Handle for running CRUD and query operations against the gallery database.
/// Holds a reference to [Database]; create with [DatabaseOperations::new].
pub struct DatabaseOperations<'a> {
    db: &'a Database,
}

impl<'a> DatabaseOperations<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    // ---------- Core CRUD ----------

    pub fn total_images(&self) -> Result<u64> {
        let conn = self.db.get_conn();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    /// Insert a single image. Returns the new row id.
    pub fn insert(&self, row: &InsertImage) -> Result<i64> {
        let conn = self.db.get_conn();
        conn.execute(
            "INSERT INTO images (path, filename) VALUES (?1, ?2)",
            [row.path.to_string_lossy().as_ref(), row.filename.as_str()],
        )?;
        Ok(conn.last_insert_rowid())
    }

    // Get one image by id. Returns [None] if not found.
    pub fn get_by_id(&self, id: i64) -> Result<Option<Image>> {
        let conn = self.db.get_conn();
        let mut stmt = conn.prepare("SELECT id, path, filename FROM images WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        match rows.next() {
            Ok(Some(row)) => Ok(Some(row_to_image(&row)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get one image by path. Returns [None] if not found.
    pub fn get_by_path(&self, path: &PathBuf) -> Result<Option<Image>> {
        let conn = self.db.get_conn();
        let mut stmt = conn.prepare("SELECT id, path, filename FROM images WHERE path = ?1")?;
        let mut rows = stmt.query([path.to_string_lossy().as_ref()])?;
        match rows.next() {
            Ok(Some(row)) => Ok(Some(row_to_image(&row)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get one image by filename. Returns [None] if not found.
    pub fn get_by_filename(&self, filename: &str) -> Result<Option<Image>> {
        let conn = self.db.get_conn();
        let mut stmt = conn.prepare("SELECT id, path, filename FROM images WHERE filename = ?1")?;
        let mut rows = stmt.query([filename])?;
        match rows.next() {
            Ok(Some(row)) => Ok(Some(row_to_image(&row)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Update an existing image by id. Only updates fields set in [UpdateImage].
    pub fn update(&self, id: i64, row: &UpdateImage) -> Result<()> {
        let conn = self.db.get_conn();
        match (&row.path, &row.filename) {
            (Some(p), Some(f)) => {
                conn.execute(
                    "UPDATE images SET path = ?1, filename = ?2 WHERE id = ?3",
                    rusqlite::params![p.to_string_lossy(), f, id],
                )?;
            }
            (Some(p), None) => {
                conn.execute("UPDATE images SET path = ?1 WHERE id = ?2", rusqlite::params![p.to_string_lossy(), id])?;
            }
            (None, Some(f)) => {
                conn.execute("UPDATE images SET filename = ?1 WHERE id = ?2", rusqlite::params![f, id])?;
            }
            (None, None) => {}
        }
        Ok(())
    }

    /// Delete one image by id. Returns whether a row was deleted.
    pub fn delete(&self, id: i64) -> Result<bool> {
        let conn = self.db.get_conn();
        let n = conn.execute("DELETE FROM images WHERE id = ?1", [id])?;
        Ok(n > 0)
    }

    // ---------- Read / Query ----------

    /// List images with optional pagination.
    pub fn list(&self, opts: &ListOptions) -> Result<Vec<Image>> {
        let conn = self.db.get_conn();
        let limit = opts.limit.max(1).min(1000);
        let mut stmt = conn.prepare(
            "SELECT id, path, filename FROM images ORDER BY id LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map([limit, opts.offset], row_to_image)?;
        rows.collect()
    }

    /// Total number of images (no filter).
    pub fn count(&self) -> Result<u64> {
        let conn = self.db.get_conn();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    /// Check if an image with the given id exists.
    // pub fn exists(&self, id: i64) -> Result<bool> {
    //     Ok(self.get_by_id(id)?.is_some())
    // }

    // ---------- Bulk ----------

    /// Insert multiple images. Returns the number of inserted rows.
    pub fn bulk_insert(&self, rows: &[InsertImage]) -> Result<usize> {
        let conn = self.db.get_conn();
        let mut stmt = conn.prepare("INSERT INTO images (path, filename) VALUES (?1, ?2)")?;
        let mut n = 0;
        for row in rows {
            stmt.execute([row.path.to_string_lossy().as_ref(), row.filename.as_str()])?;
            n += 1;
        }
        Ok(n)
    }

    /// Insert or replace by path (UNIQUE). Uses SQLite INSERT OR REPLACE.
    pub fn upsert(&self, row: &InsertImage) -> Result<()> {
        let conn = self.db.get_conn();
        conn.execute(
            "INSERT INTO images (path, filename) VALUES (?1, ?2) ON CONFLICT(path) DO UPDATE SET filename = excluded.filename",
            [row.path.to_string_lossy().as_ref(), row.filename.as_str()],
        )?;
        Ok(())
    }

    /// Delete multiple images by id. Returns the number of deleted rows.
    pub fn bulk_delete(&self, ids: &[i64]) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.db.get_conn();
        let placeholders = ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM images WHERE id IN ({})", placeholders);
        let mut stmt = conn.prepare(&sql)?;
        let n = stmt.execute(rusqlite::params_from_iter(ids.iter()))?;
        Ok(n)
    }

        
}

fn row_to_image(row: &rusqlite::Row<'_>) -> Result<Image> {
    Ok(Image {
        id: row.get(0)?,
        path: PathBuf::from(row.get::<_, String>(1)?),
        filename: row.get(2)?,
    })
}
