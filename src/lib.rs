#![warn(missing_docs)]

//! A `Pinboard` is a shared, mutable, eventually consistent, lock-free data-structure.  This
//! allows multiple threads to communicate in a decoupled way by publishing data to the pinboard
//! which other threads can then read in an eventually consistent way.
//!
//! This is not a silver bullet though, there are various limitations of `Pinboard` that trade off
//! the nice behaviour described above.
//!
//! * Eventual consistency:
//!     * Writes from one thread are not guaranteed to be seen by reads from another thread
//!     * Writes from one thread can overwrite writes from another thread
//! * No in-place mutation:
//!     * The only write primitive completely overwrites the data on the `Pinboard`
//! * Requires `Clone`:
//!     * All reads return a clone of the data, decoupling the lifetime of the read value from the
//!     data stored in the global reference.

extern crate crossbeam_epoch as epoch;

use epoch::{Atomic, Owned, Shared, pin};
use std::sync::atomic::Ordering::*;

/// An instance of a `Pinboard`, holds a shared, mutable, eventually-consistent reference to a `T`.
pub struct Pinboard<T: Clone + 'static>(Atomic<T>);

impl<T: Clone + 'static> Pinboard<T> {
    /// Create a new `Pinboard` instance holding the given value.
    pub fn new(t: T) -> Pinboard<T> {
        let t = Owned::new(t);
        let p = Pinboard::default();
        p.0.store(t, Release);
        p
    }

    /// Update the value stored in the `Pinboard`.
    pub fn set(&self, t: T) {
        let guard = pin();
        let t = Owned::new(t);
        let t = self.0.swap(t, AcqRel, &guard);
        unsafe {
            if !t.is_null() {
                guard.defer_unchecked(move || drop(t.into_owned()));
            }
        }
    }

    /// Clear out the `Pinboard` so its no longer holding any data.
    pub fn clear(&self) {
        let guard = pin();
        let t = self.0.swap(Shared::null(), AcqRel, &guard);
        unsafe {
            if !t.is_null() {
                guard.defer_unchecked(move || drop(t.into_owned()));
            }
        }
    }

    /// Get a copy of the latest (well, recent) version of the posted data.
    pub fn read(&self) -> Option<T> {
        let guard = pin();
        unsafe {
            let t = self.0.load(Acquire, &guard);
            if t.is_null() {
                None
            } else {
                Some(t.deref().clone())
            }
        }
    }
}

impl<T: Clone + 'static> Default for Pinboard<T> {
    fn default() -> Pinboard<T> {
        Pinboard(Atomic::null())
    }
}

impl<T: Clone + 'static> Drop for Pinboard<T> {
    fn drop(&mut self) {
        // Make sure any stored data is marked for deletion
        self.clear();
    }
}

impl<T: Clone + 'static> From<Option<T>> for Pinboard<T> {
    fn from(src: Option<T>) -> Pinboard<T> {
        src.map(Pinboard::new).unwrap_or_default()
    }
}

/// An wrapper around a `Pinboard` which provides the guarantee it is never empty.
pub struct NonEmptyPinboard<T: Clone + 'static>(Pinboard<T>);

impl<T: Clone + 'static> NonEmptyPinboard<T> {
    /// Create a new `NonEmptyPinboard` instance holding the given value.
    pub fn new(t: T) -> NonEmptyPinboard<T> {
        NonEmptyPinboard(Pinboard::new(t))
    }

    /// Update the value stored in the `NonEmptyPinboard`.
    #[inline]
    pub fn set(&self, t: T) {
        self.0.set(t)
    }

    /// Get a copy of the latest (well, recent) version of the posted data.
    #[inline]
    pub fn read(&self) -> T {
        // Unwrap the option returned by the inner `Pinboard`. This will never panic, because it's
        // impossible for this `Pinboard` to be empty (though it's not possible to prove this to the
        // compiler).
        match self.0.read() {
            Some(t) => t,
            None => unreachable!("Inner pointer was unexpectedly null"),
        }
    }
}

macro_rules! debuggable {
    ($struct:ident, $trait:ident) => {
        impl<T: Clone + 'static> ::std::fmt::$trait for $struct<T> where T: ::std::fmt::$trait {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                write!(f, "{}(", stringify!($struct))?;
                ::std::fmt::$trait::fmt(&self.read(), f)?;
                write!(f, ")")
            }
        }
    }
}

debuggable!(Pinboard, Debug);
debuggable!(NonEmptyPinboard, Debug);
debuggable!(NonEmptyPinboard, Binary);
debuggable!(NonEmptyPinboard, Display);
debuggable!(NonEmptyPinboard, LowerExp);
debuggable!(NonEmptyPinboard, LowerHex);
debuggable!(NonEmptyPinboard, Octal);
debuggable!(NonEmptyPinboard, Pointer);
debuggable!(NonEmptyPinboard, UpperExp);
debuggable!(NonEmptyPinboard, UpperHex);

#[cfg(test)]
mod tests {
    extern crate crossbeam;
    use super::*;
    use std::fmt::Display;

    fn consume<T: Clone + Display>(t: &Pinboard<T>) {
        loop {
            match t.read() {
                Some(_) => {}
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

    fn check_debug<T: ::std::fmt::Debug>(_: T) {}

    #[test]
    fn it_works() {
        let t = Pinboard::<u32>::default();
        assert_eq!(None, t.read());
        t.set(3);
        assert_eq!(Some(3), t.read());
        t.clear();
        assert_eq!(None, t.read());
    }

    #[test]
    fn single_producer_single_consumer() {
        let t = Pinboard::<u32>::new(0);

        crossbeam::scope(|scope| {
            scope.spawn(|| produce(&t));
            scope.spawn(|| consume(&t));
        })
    }

    #[test]
    fn multi_producer_single_consumer() {
        let t = Pinboard::<u32>::new(0);

        crossbeam::scope(|scope| {
            scope.spawn(|| produce(&t));
            scope.spawn(|| produce(&t));
            scope.spawn(|| produce(&t));
            scope.spawn(|| consume(&t));
        })
    }

    #[test]
    fn single_producer_multi_consumer() {
        let t = Pinboard::<u32>::new(0);

        crossbeam::scope(|scope| {
            scope.spawn(|| produce(&t));
            scope.spawn(|| consume(&t));
            scope.spawn(|| consume(&t));
            scope.spawn(|| consume(&t));
        })
    }

    #[test]
    fn multi_producer_multi_consumer() {
        let t = Pinboard::<u32>::new(0);

        crossbeam::scope(|scope| {
            scope.spawn(|| produce(&t));
            scope.spawn(|| produce(&t));
            scope.spawn(|| produce(&t));
            scope.spawn(|| consume(&t));
            scope.spawn(|| consume(&t));
            scope.spawn(|| consume(&t));
        })
    }

    #[test]
    fn non_empty_pinboard() {
        let t = NonEmptyPinboard::<u32>::new(3);
        assert_eq!(3, t.read());
        t.set(4);
        assert_eq!(4, t.read());
    }

    #[test]
    fn debuggable() {
        let t = Pinboard::<i32>::new(3);
        check_debug(t);
        let t = NonEmptyPinboard::<i32>::new(2);
        check_debug(t);
    }
}
