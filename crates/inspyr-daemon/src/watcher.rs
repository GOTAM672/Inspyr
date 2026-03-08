use notify::{Event, EventKind, RecursiveMode, Result, Watcher};
use std::{
    path::Path,
    sync::mpsc,
};
use walkdir::{DirEntry, WalkDir};

pub struct FileWatcher;

impl FileWatcher {
    pub fn start_watcher(watch_dir: &Path) -> Result<()> {
        let (tx, rx) = mpsc::channel::<Result<Event>>();

        let mut watcher = notify::recommended_watcher(tx)?;

        println!("Starting watcher on: {}", watch_dir.display());

        for entry in WalkDir::new(watch_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !is_hidden_dir(e))
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
                Ok(event) => handle_event(event),
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        }

        Ok(())
    }
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
        if is_hidden_path(&path) {
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
