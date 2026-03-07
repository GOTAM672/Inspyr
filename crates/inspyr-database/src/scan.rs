//! Scan directory and update database. Stub for initial/on-demand scan.

use crate::{Database, DatabaseOperations, InsertImage};
use rusqlite::Result;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// Scan a directory and sync media into the database.
/// Used for initial or on-demand full scan.

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff"];

pub struct Scan<'a> {
    db: &'a Database,
}

impl<'a> Scan<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Walk the directory tree, skip hidden files/dirs, and insert image files into the database.
    pub fn scan_directory(&self, path: &Path) -> Result<()> {
        let ops = DatabaseOperations::new(self.db);
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| !Self::is_hidden_entry(e))
            .filter_map(|e: Result<DirEntry, walkdir::Error>| e.ok())
        {
            if entry.file_type().is_file() && Self::is_image_file(entry.path()) {
                let path_buf = entry.path().to_path_buf();
                let filename = entry
                    .file_name()
                    .to_str()
                    .unwrap_or("")
                    .to_string();
                ops.insert(&InsertImage {
                    path: path_buf,
                    filename,
                })?;
            }
        }
        Ok(())
    }

    // pub fn re_scan_directory(&self, path: &Path) -> Result<()> {
    //     for entry in WalkDir::new(path) {
    //         let entry = entry.unwrap();
    //         println!("{}", entry.path().display());
    //     }
    //     Ok(())
    // }

    // fn is_directory(path: &Path) -> bool {
    //     path.is_dir()
    // }


    /// Skip hidden files and directories (name starts with `.`). Used so we don't descend into hidden dirs.
    fn is_hidden_entry(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }

    fn is_hidden_dir(entry: &DirEntry) -> bool {
        if entry.depth() == 0 {
            return false;
        }
        if entry.file_type().is_dir() {
            entry
                .file_name()
                .to_str()
                .map(|name| name.starts_with('.'))
                .unwrap_or(false)
        } else {
            false
        }
    }

    fn is_hidden_path(path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }

    fn is_image_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                IMAGE_EXTENSIONS
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case(ext))
            })
            .unwrap_or(false)
    }
}
