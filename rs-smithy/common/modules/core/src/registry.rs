use std::collections::HashMap;
use std::ops::Deref;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::borrow::{Cow, Borrow};



// Thread-safe store for resources
// The registry will appear as immutable except for 'update' operations

#[derive(Clone)]
pub struct GlobalRegistry<'a, K: Eq + Hash, V: Clone> {
    inner: Arc<RwLock<HashMap<K, Cow<'a, V>>>>,
}

impl<'a, K: Eq + Hash, V: Clone> GlobalRegistry<'a, K, V> {
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn get<Q>(&self, k: &Q) -> Option<Cow<'a, V>> 
    where
        Q: ?Sized + Eq + Hash,
        K: Borrow<Q>,
    {
        self.inner.read().unwrap().get(k).map(Cow::clone)
    }

    pub fn insert(&self, k: K, v: V) {
        self.inner.write().unwrap().insert(k, Cow::Owned(v));
    }
}

/* 
impl GlobalRegistry {
    type Key;
    type Value;

    fn new() -> Self {
        Self { store: HashMap::new() }
    }

    fn get(&self, key: K) -> Option<&V> {
        self.store.lock().unwrap().get(&key) // This lock should only ever be held internally
    }
}
*/

//struct GR

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path() {
        let r: GlobalRegistry<String, String> = GlobalRegistry::new();
        assert_eq!(r.get("foo"), None);
        r.insert("foo".to_owned(), "bar".to_owned());
        let bar = "bar".to_owned();
        assert!(matches!(r.get("foo"), Some(Cow::Owned(bar))));
    }

    #[tokio::test]
    async fn my_test() {
        let r: GlobalRegistry<usize, usize> = GlobalRegistry::new();

        let r1 = r.clone();
        let t1 = tokio::spawn(async move {
            r1.insert(0, 1);
            assert!(matches!(r1.get(&0), Some(Cow::Owned(1))));
            r1.insert(1, 2);
        });

        let r2 = r.clone();
        let t2 = tokio::spawn(async move {
            assert!(matches!(r2.get(&0), None));
        });

        let t3 = tokio::spawn(async move {

        });

        tokio::try_join!(t1, t2, t3).unwrap();
    }
}
