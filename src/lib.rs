extern crate crossbeam;

use crossbeam::mem::epoch::{Atomic, Owned, pin};
use std::sync::atomic::Ordering::*;

use std::ops::Deref;

pub struct Pinboard<T: Clone>(Atomic<T>);

impl<T: Clone> Pinboard<T> {
    pub fn new() -> Pinboard<T> {
        Pinboard(Atomic::null())
    }

    pub fn set(&self, t: T) {
        let guard = pin();
        let t = Owned::new(t);
        if let Some(t) = self.0.swap(Some(t), Release, &guard) {
            unsafe { guard.unlinked(t); }
        }
    }

    pub fn clear(&self) {
        let guard = pin();
        if let Some(t) = self.0.swap(None, Release, &guard) {
            unsafe { guard.unlinked(t); }
        }
    }

    pub fn read(&self) -> Option<T> {
        let guard = pin();
        let t = self.0.load(Acquire, &guard);
        t.map(|t| -> &T { t.deref() }).cloned()
    }
}

impl<T: Clone> Drop for Pinboard<T> {
    fn drop(&mut self) {
        // Make sure any stored data is marked for deletion
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Display;

    fn consume<T: Clone + Display>(t: &Pinboard<T>) {
        loop {
            match t.read() {
                Some(_) => {},
                None => break,
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    fn produce(t: &Pinboard<u32>) {
        for i in 1..100 {
            t.set(i);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        t.clear();
    }

    #[test]
    fn it_works() {
        let t = Pinboard::<u32>::new();
        assert_eq!(None, t.read());
        t.set(3);
        assert_eq!(Some(3), t.read());
        t.clear();
        assert_eq!(None, t.read());
    }

    #[test]
    fn single_producer_single_consumer() {
        let t = Pinboard::<u32>::new();
        t.set(0);

        crossbeam::scope(|scope| {
            scope.spawn(|| produce(&t));
            scope.spawn(|| consume(&t));
        })
    }

    #[test]
    fn multi_producer_single_consumer() {
        let t = Pinboard::<u32>::new();
        t.set(0);

        crossbeam::scope(|scope| {
            scope.spawn(|| produce(&t));
            scope.spawn(|| produce(&t));
            scope.spawn(|| produce(&t));
            scope.spawn(|| consume(&t));
        })
    }

    #[test]
    fn single_producer_multi_consumer() {
        let t = Pinboard::<u32>::new();
        t.set(0);

        crossbeam::scope(|scope| {
            scope.spawn(|| produce(&t));
            scope.spawn(|| consume(&t));
            scope.spawn(|| consume(&t));
            scope.spawn(|| consume(&t));
        })
    }

    #[test]
    fn multi_producer_multi_consumer() {
        let t = Pinboard::<u32>::new();
        t.set(0);

        crossbeam::scope(|scope| {
            scope.spawn(|| produce(&t));
            scope.spawn(|| produce(&t));
            scope.spawn(|| produce(&t));
            scope.spawn(|| consume(&t));
            scope.spawn(|| consume(&t));
            scope.spawn(|| consume(&t));
        })
    }
}
