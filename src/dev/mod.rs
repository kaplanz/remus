//! Generic memory mapped device.
//!
//! # Usage
//!
//! The [`Device`] trait is useful in combination with [`Bus`](crate::bus::Bus).
//! Together, they can be used to emulate the behaviour of [memory-mapped I/O].
//!
//! [memory-mapped I/O]: https://en.wikipedia.org/wiki/Memory-mapped_I/O

use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

pub use self::null::Null;
pub use self::random::Random;
use crate::blk::Block;
use crate::mem::Memory;

mod null;
mod random;

pub type SharedDevice = Rc<RefCell<dyn Device>>;

/// Memory-mapped I/O device.
pub trait Device: Block {
    /// Checks if the device contains the provided `index` within its
    /// address space.
    fn contains(&self, index: usize) -> bool;

    /// Returns the length of the device, in bytes.
    fn len(&self) -> usize;

    /// Checks if the length of the device is zero.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reads a byte from the specified address.
    fn read(&self, index: usize) -> u8;

    /// Writes a byte to the specified address.
    fn write(&mut self, index: usize, value: u8);

    /// Constructs a `SharedDevice` from `self`.
    fn to_shared(self) -> SharedDevice
    where
        Self: 'static + Sized,
    {
        Rc::new(RefCell::new(self))
    }
}

impl<T> Device for T
where
    T: Block + DerefMut + Memory,
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
    use crate::mem::Ram;

    #[test]
    fn device_contains_works() {
        (0..0x100).for_each(|index| assert!(Device::contains(&Ram::from(&[0xaau8; 0x100]), index)));
    }

    #[test]
    fn device_read_works() {
        (0..0x100).for_each(|index| assert_eq!(Ram::from(&[0xaau8; 0x100]).read(index), 0xaa));
    }

    #[test]
    fn device_write_works() {
        let mut dev = Ram::from(&[0u8; 0x100]);
        (0..0x100).for_each(|index| dev.write(index, 0xaa));
        (0..0x100).for_each(|index| assert_eq!(Ram::from(&[0xaau8; 0x100]).read(index), 0xaa));
    }
}
