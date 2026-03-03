use rusqlite::{Connection, Error, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Database {
    conn: Connection,
    scan_dir: PathBuf,
    db_path: PathBuf,
}

impl Database {
    pub fn init() -> Result<Self> {
        let scan_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));

        let db_dir = scan_dir.join(".inspyr");
        fs::create_dir_all(&db_dir).map_err(|e| Error::ToSqlConversionFailure(Box::new(e)))?;

        let db_path = db_dir.join("gallery.db");
        let conn = Connection::open(&db_path)?;
        let db = Database {
            conn,
            scan_dir,
            db_path,
        };

        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        let sql = include_str!("tables.sql");
        self.conn.execute_batch(sql)?;
        Ok(())
    }

    pub fn is_database_empty(&self) -> bool {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))
            .unwrap_or(0);
        count == 0
    }

    pub(crate) fn get_conn(&self) -> &Connection {
        &self.conn
    }

    pub fn get_db_path(&self) -> PathBuf {
        self.db_path.clone()
    }

    pub fn get_scan_dir(&self) -> PathBuf {
        self.scan_dir.clone()
    }
}
