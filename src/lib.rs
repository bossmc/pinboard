extern crate crossbeam;

use crossbeam::mem::epoch::{Atomic, Owned, pin};
use std::sync::atomic::Ordering::*;

use std::ops::Deref;

pub struct Shelf<T: Clone>(Atomic<T>);

impl<T: Clone> Shelf<T> {
    pub fn new() -> Shelf<T> {
        Shelf(Atomic::null())
    }

    pub fn set(&self, t: T) {
        let t = Owned::new(t);
        self.0.store(Some(t), Release);
    }

    pub fn clear(&self) {
        self.0.store(None, Release);
    }

    pub fn read(&self) -> Option<T> {
        let guard = pin();
        let t = self.0.load(Acquire, &guard);
        t.map(|t| -> &T { t.deref() }).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Display;

    fn consume<T: Clone + Display>(t: &Shelf<T>) {
        loop {
            match t.read() {
                Some(_) => {},
                None => break,
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    fn produce(t: &Shelf<u32>) {
        for i in 1..100 {
            t.set(i);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        t.clear();
    }

    #[test]
    fn it_works() {
        let t = Shelf::<u32>::new();
        assert_eq!(None, t.read());
        t.set(3);
        assert_eq!(Some(3), t.read());
        t.clear();
        assert_eq!(None, t.read());
    }

    #[test]
    fn single_producer_single_consumer() {
        let t = Shelf::<u32>::new();
        t.set(0);

        crossbeam::scope(|scope| {
            scope.spawn(|| produce(&t));
            scope.spawn(|| consume(&t));
        })
    }

    #[test]
    fn multi_producer_single_consumer() {
        let t = Shelf::<u32>::new();
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
        let t = Shelf::<u32>::new();
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
        let t = Shelf::<u32>::new();
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
