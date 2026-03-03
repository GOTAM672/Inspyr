mod watcher;

use glib::MainLoop;
use inspyr_database::Database;

use crate::watcher::FileWatcher;

fn main() {
    let main_loop = MainLoop::new(None, false);

    println!("Starting Inspyr daemon...");

    let db = Database::init().expect("Failed to initialize database");

    println!("Database initialized successfully");
    println!("Database path: {:?}", db.get_db_path());
    println!("Scan directory: {:?}", db.get_scan_dir());

    if !db.is_database_empty() {
        println!("Database is not empty...");
    }

    FileWatcher::start_watcher(&db.get_scan_dir()).unwrap();

    main_loop.run();
}
