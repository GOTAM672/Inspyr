mod database;

use gtk::glib::MainLoop;

use self::database::Database;

fn main() {
    let main_loop = MainLoop::new(None, false);

    println!("Starting Inspyr daemon...");

    let db = Database::init().expect("Failed to initialize database");

    println!("Database initialized successfully");
    println!("Database path: {:?}", db.get_db_path());
    println!("Home directory: {:?}", db.get_home_dir());

    // Run daemon loop
    main_loop.run();
}