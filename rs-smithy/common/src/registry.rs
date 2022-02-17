// File registry construct
use std::collections::{HashMap};
use std::sync::{Arc};
use tokio::sync::broadcast::{channel, Sender, Receiver};

enum EventKind {
    Create,
    Update,
    Delete,
    // Move ?
}


struct FileChangeEvent<T: Send> {
    kind: EventKind,
    data: Arc<T>,
}

pub struct FileRegistry<T> {
    entries: HashMap<String, Entry<T>>,
    emitter: Sender<Arc<T>>,
}

pub struct Entry<T> {
    path: String,
    data: Arc<T>, // use Arc?
    stat: String,
    // TODO: store info for caching
}

impl<T> Entry<T> {
    pub fn new(path: String, data: T, stat: String) -> Self {
        Entry { path, stat, data: Arc::new(data) }
    }
}

impl<T> FileRegistry<T> {
    pub fn new() -> Self {
        let (emitter, _) = channel::<Arc<T>>(128);

        FileRegistry { 
            emitter,
            entries: HashMap::new(), 
        }
    }

    pub fn add(&mut self, entry: Entry<T>) {
        self.notify(&entry);
        self.entries.insert(entry.path.to_owned(), entry);
    }

    pub fn get(&self, path: &str) -> Option<&Entry<T>> {
        self.entries.get(path)
    }

    pub fn subscribe(&self) -> Receiver<Arc<T>> {
        self.emitter.subscribe()
    }

    // should be private
    fn notify(&mut self, entry: &Entry<T>) {
        match self.emitter.send(entry.data.clone()) {
            Ok(v) => println!("Send message: {}", v),
            Err(err) => {},
        }
    }
}
