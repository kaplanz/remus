//! Generic memory mapped device.
//!
//! # Usage
//!
//! The [`Device`] trait is useful in combination with [`Bus`](crate::bus::Bus).
//! Together, they can be used to emulate the behaviour of [memory-mapped I/O].
//!
//! [memory-mapped I/O]: https://en.wikipedia.org/wiki/Memory-mapped_I/O

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

pub use self::null::Null;
pub use self::random::Random;

mod null;
mod random;

/// Memory-mapped I/O device.
pub trait Device: Debug {
    fn len(&self) -> usize;

    fn read(&self, index: usize) -> u8;

    fn write(&mut self, index: usize, value: u8);

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Device for T
where
    T: Debug + Deref<Target = [u8]> + DerefMut,
{
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }

    fn read(&self, index: usize) -> u8 {
        self[index]
    }

    fn write(&mut self, index: usize, value: u8) {
        self[index] = value;
    }
}
