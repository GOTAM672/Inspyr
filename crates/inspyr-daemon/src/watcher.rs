use glib::subclass::prelude::*;
use notify::{Event, EventKind, RecursiveMode, Result, Watcher};
use std::{
    path::Path,
    sync::mpsc,
};
use walkdir::{DirEntry, WalkDir};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct FileWatcher;

    #[glib::object_subclass]
    impl ObjectSubclass for FileWatcher {
        const NAME: &'static str = "FileWatcher";
        type Type = super::FileWatcher;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for FileWatcher {}
}

glib::wrapper! {
    pub struct FileWatcher(ObjectSubclass<imp::FileWatcher>);
}

impl Default for FileWatcher {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

impl FileWatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_watcher(&self, watch_dir: &Path) -> Result<()> {
        let (tx, rx) = mpsc::channel::<Result<Event>>();

        let mut watcher = notify::recommended_watcher(tx)?;

        println!("Starting watcher on: {}", watch_dir.display());

        for entry in WalkDir::new(watch_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !Self::is_hidden_dir(e))
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_dir() {
                if let Err(e) = watcher.watch(path, RecursiveMode::NonRecursive) {
                    eprintln!(
                        "Skipping directory (watch error): {} -> {:?}",
                        path.display(),
                        e
                    );
                }
            }
        }

        println!("Watching started successfully.");

        for res in rx {
            match res {
                Ok(event) => Self::handle_event(event),
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        }

        Ok(())
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

    fn handle_event(event: Event) {
        for path in event.paths {
            if Self::is_hidden_path(&path) {
                continue;
            }

            match event.kind {
                EventKind::Create(_) => {
                    println!("Create Event: {:?} -> {}", event.kind, path.display());
                }
                EventKind::Modify(_) => {
                    println!("Modify Event: {:?} -> {}", event.kind, path.display());
                }
                EventKind::Remove(_) => {
                    println!("Remove Event: {:?} -> {}", event.kind, path.display());
                }
                EventKind::Other => {
                    println!("Other Event: {:?} -> {}", event.kind, path.display());
                }
                notify::EventKind::Any | notify::EventKind::Access(_) => {}
            }
        }
    }
}
