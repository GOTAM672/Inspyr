use rusqlite::{Connection, Result, Error};

use std::path::PathBuf;
use std::fs;

#[derive(Debug)]
pub struct Database {
    conn: Connection,
    home_dir: PathBuf,
    db_path: PathBuf,
}

// Image file extensions
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png"];

impl Database {
    pub fn init() -> Result<Self> {
        let home_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"));

        // Create .inspyr directory in home if it doesn't exist
        let db_dir = home_dir.join(".inspyr");
        fs::create_dir_all(&db_dir)
            .map_err(|e| Error::ToSqlConversionFailure(Box::new(e)))?;
        
        let db_path = db_dir.join("gallery.db");
        println!("Database path: {:?}", db_path);
        let conn = Connection::open(&db_path)?;
        
        let db = Database { conn, home_dir, db_path };

        db.create_tables()?;

        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        // Read the SQL file
        let sql = include_str!("tables.sql");
         
        // Execute the SQL statements
        self.conn.execute_batch(sql)?;
        
        Ok(())
    }

    pub fn get_db_path(&self) -> PathBuf {
        self.db_path.clone()
    }

    pub fn get_home_dir(&self) -> PathBuf {
        self.home_dir.clone()
    }
}