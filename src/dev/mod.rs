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
use crate::blk::Block;

mod null;
mod random;

/// Memory-mapped I/O device.
pub trait Device: Block + Debug {
    /// Check if the [`Device`] contains the provided index within its address
    /// space for performing [`read`](Device::read)s and
    /// [`write`](Device::write)s.
    fn contains(&self, index: usize) -> bool;

    /// Returns the length of the [`Device`], in bytes.
    fn len(&self) -> usize;

    /// Check if the length of the [`Device`] is zero.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Perform a read of the byte at the specified address.
    fn read(&self, index: usize) -> u8;

    /// Perform a write to the byte at the specified address.
    fn write(&mut self, index: usize, value: u8);
}

impl<T> Device for T
where
    T: Block + Debug + Deref<Target = [u8]> + DerefMut,
{
    fn contains(&self, index: usize) -> bool {
        (0..self.len()).contains(&index)
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_contains_works() {
        (0..0x100).for_each(|index| assert!(Device::contains(&vec![0xaau8; 0x100], index)));
    }

    #[test]
    fn device_read_works() {
        (0..0x100).for_each(|index| assert_eq!(vec![0xaau8; 0x100].read(index), 0xaa));
    }

    #[test]
    fn device_write_works() {
        let mut dev = vec![0u8; 0x100];
        (0..0x100).for_each(|index| dev.write(index, 0xaa));
        (0..0x100).for_each(|index| assert_eq!(vec![0xaau8; 0x100].read(index), 0xaa));
    }
}
