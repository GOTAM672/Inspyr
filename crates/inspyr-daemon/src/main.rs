mod watcher;

use glib::MainLoop;
use inspyr_database::{Database, DatabaseOperations, Scan};
use std::path::Path;

use crate::watcher::FileWatcher;

fn main() {
    let main_loop = MainLoop::new(None, false);

    println!("Starting Inspyr daemon...");

    let db = Database::init().expect("Failed to initialize database");

    println!("Database initialized successfully");
    println!("Database path: {:?}", db.get_db_path());
    println!("Scan directory: {:?}", db.get_scan_dir());

    if db.is_database_empty() {
        println!("Database is empty. Starting initial scan...");
        let scan = Scan::new(&db);
        scan.initial_scan(&db.get_scan_dir())
            .expect("Failed to scan directory");

        let db_ops = DatabaseOperations::new(&db);
        let total_images = db_ops.total_images().expect("Failed to get total images");
        println!("Total images inserted: {}", total_images);
    }

    println!("Starting re-scan...");
    let scan = Scan::new(&db);
    let db_ops = DatabaseOperations::new(&db);
    scan.re_scan(&Path::new("/home/gautham/Demoimage"))
        .expect("Failed to re-scan directory");
    let total_images = db_ops.total_images().expect("Failed to get total images");
    println!("Total images re-inserted: {}", total_images);

    FileWatcher::new()
        .start_watcher(&db.get_scan_dir())
        .unwrap();

    main_loop.run();
}
