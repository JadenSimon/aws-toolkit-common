// Watch for SAM templates
// That's about it.
use std::sync::{mpsc::{channel}, Arc};
use notify::{Op, Watcher, RawEvent, RecursiveMode, FsEventWatcher, DebouncedEvent};
use tokio::sync::Mutex;
use yaml_rust::{Yaml, YamlLoader, YamlEmitter};

use crate::registry::{Entry, FileRegistry};

pub struct FileWatcher {
    pub handle: tokio::task::JoinHandle<()>,
    pub watcher: FsEventWatcher,
}

type Event = DebouncedEvent;

/// https://docs.rs/notify/latest/notify/struct.RawEvent.html
pub fn handle_event(event: Event, reg: Arc<Mutex<FileRegistry<Yaml>>>) {
    // note that RENAME is fired twice for moves
    // would need to store state to check for cookie

    match event {
        DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
            // Seems to fire a `Write` + `NoticeWrite` at the same time on Mac ??
            tokio::spawn(async move {
                let contents = tokio::fs::read(&path).await;
                match contents {
                    Ok(bytes) => {
                        let mut yaml = YamlLoader::load_from_str(std::str::from_utf8(&bytes).unwrap()).unwrap();
                        let mut locked = reg.lock().await;
                        // just ignore more than 1 doc
                        if yaml.len() == 0 {
                            return Ok(()); // I mean it's really an error but eh
                        }
                        let doc = yaml.remove(0);
                        locked.add(Entry::new(path.to_str().unwrap().to_string(), doc, "".to_string()));
                        Ok(())
                    },
                    Err(err) => {
                        println!("{:?}", err);
                        Err(err)
                    },
                }
            });
        },
        DebouncedEvent::Error(err, path) => {

        },
        _ => {},
    }
}

pub fn start(directory: &str) -> FileWatcher {
    let (tx, rx) = channel::<Event>();
    // Seems like the debounce value needs to be tweaked. If it's too low you get double emits
    let mut watcher: FsEventWatcher = Watcher::new(tx, std::time::Duration::from_millis(25)).unwrap();

    watcher.watch(directory, RecursiveMode::Recursive).unwrap();

    let reg = crate::registry::FileRegistry::<Yaml>::new();
    let mut listener = reg.subscribe();

    tokio::spawn(async move {
        loop {
            match listener.recv().await {
                Ok(yaml) => {
                    // Check for SAM templates
                    let transform = yaml["Transform"].as_str().unwrap_or("");
                    if transform.starts_with("AWS::Serverless") {
                        println!("Found SAM template!");
                    }
                },
                Err(err) => println!("{:?}", err)
            }
        }
    });

    let handle = tokio::task::spawn_blocking(move || {
        let threaded = Arc::new(Mutex::new(reg));
        loop {
            match rx.recv() {
                Ok(event) => {
                    println!("{:?}", event);
                    handle_event(event, Arc::clone(&threaded));
                },
                Err(err) =>  println!("{:?}", err),
            }
        }
    });

    FileWatcher { handle, watcher }
}