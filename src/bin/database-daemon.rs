mod database;

use self::database::Database;

fn main () {
    println!("Hello, world!");
    let _db = Database::init().expect("Failed to initialize database");
    println!("Database initialized successfully");
    println!("Database path: {:?}", _db.get_db_path());
    println!("Home directory: {:?}", _db.get_home_dir());
}