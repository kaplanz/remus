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
    /// Check if the [`Device`] contains the provided index within its address
    /// space for performing [`read`](Device::read)s and
    /// [`write`](Device::write)s.
    fn contains(&self, index: usize) -> bool;

    /// Perform a read of the byte at the specified address.
    fn read(&self, index: usize) -> u8;

    /// Perform a write to the byte at the specified address.
    fn write(&mut self, index: usize, value: u8);
}

impl<T> Device for T
where
    T: Debug + Deref<Target = [u8]> + DerefMut,
{
    fn contains(&self, index: usize) -> bool {
        (0..<[u8]>::len(self)).contains(&index)
    }

    fn read(&self, index: usize) -> u8 {
        self[index]
    }

    fn write(&mut self, index: usize, value: u8) {
        self[index] = value;
    }
}
