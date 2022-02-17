use crate::parser::{Parser, ConfigNode};
use std::sync::Arc;
use notify::{Op, Watcher, RawEvent, RecursiveMode, FsEventWatcher, DebouncedEvent};
use tokio::sync::broadcast::{Sender, Receiver};

// Basic watcher for credential files
// This should be in another file?

pub struct FileWatcher {
    pub handle: tokio::task::JoinHandle<()>,
    pub watcher: FsEventWatcher,
    emitter: Sender<Arc<Vec<ConfigNode>>>,
}

type Event = DebouncedEvent;

/// https://docs.rs/notify/latest/notify/struct.RawEvent.html
pub fn handle_event(event: Event, emitter: Sender<Arc<Vec<ConfigNode>>>) {
    // note that RENAME is fired twice for moves
    // would need to store state to check for cookie

    match event {
        DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
            // Seems to fire a `Write` + `NoticeWrite` at the same time on Mac ??
            tokio::spawn(async move {
                let contents = tokio::fs::read(&path).await;
                match contents {
                    Ok(bytes) => {
                        let parser = Parser::new(std::str::from_utf8(&bytes).unwrap().to_string());
                        let parsed = parser.parse_file();
                        emitter.send(Arc::new(parsed)).unwrap();
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
    let (tx, rx) = std::sync::mpsc::channel::<Event>();
    // Seems like the debounce value needs to be tweaked. If it's too low you get double emits
    let mut watcher: FsEventWatcher = Watcher::new(tx, std::time::Duration::from_millis(25)).unwrap();

    watcher.watch(directory, RecursiveMode::Recursive).unwrap();

    let (emitter, _) = tokio::sync::broadcast::channel::<Arc<Vec<ConfigNode>>>(100);

    let copy = emitter.clone();
    let handle = tokio::task::spawn_blocking(move || {
        loop {
            match rx.recv() {
                Ok(event) => {
                    handle_event(event, copy.clone());
                },
                Err(err) =>  println!("{:?}", err),
            }
        }
    });

    FileWatcher { handle, watcher, emitter }
}

impl FileWatcher {
    pub fn subscribe(&self) -> Receiver<Arc<Vec<ConfigNode>>> {
        self.emitter.subscribe()
    }
}