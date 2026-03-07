mod watcher;

use glib::MainLoop;
use std::path::{Path, PathBuf};
use inspyr_database::{Database, DatabaseOperations, InsertImage, Scan};

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
        scan.scan_directory(&db.get_scan_dir()).expect("Failed to scan directory");

        let db_ops = DatabaseOperations::new(&db);
        let total_images = db_ops.total_images().expect("Failed to get total images");
        println!("Total images inserted: {}", total_images);
    }

    // let db_ops = DatabaseOperations::new(&db);

    // let image = InsertImage { path: PathBuf::from("/home/gautham/Pictures/test1.jpg"), filename: "test.jpg".to_string() };

    // db_ops.insert(&image).expect("Failed to insert image");
    // let image1 = db_ops.get_by_id(1).unwrap();
    // println!("Image: {:?}", image1);

    // let image2 = db_ops.get_by_path(&image.path).unwrap();
    // println!("Image: {:?}", image2);

    // let image3 = db_ops.get_by_filename(&image.filename).unwrap();
    // println!("Image: {:?}", image3);


    FileWatcher::start_watcher(&db.get_scan_dir()).unwrap();

    main_loop.run();
}
