//! Generic memory mapped device.
//!
//! # Usage
//!
//! The [`Device`] trait is useful in combination with [`Bus`](crate::bus::Bus).
//! Together, they can be used to emulate the behaviour of [memory-mapped I/O].
//!
//! [memory-mapped I/O]: https://en.wikipedia.org/wiki/Memory-mapped_I/O

use crate::arch::Address;
use crate::blk::Block;
use crate::share::Shared;

mod null;
mod random;

pub use self::null::Null;
pub use self::random::Random;

/// Memory-mapped I/O device.
pub trait Device: Address<u8> + Block {
    /// Checks if the device contains the provided `index` within its
    /// address space.
    fn contains(&self, index: usize) -> bool;

    /// Returns the length of the device, in bytes.
    fn len(&self) -> usize;

    /// Checks if the length of the device is zero.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Constructs a [`Shared`] device from `self`.
    fn to_shared(self) -> Shared<Self>
    where
        Self: 'static + Sized,
    {
        self.into()
    }

    /// Constructs a [`Dynamic`] device from `self`.
    fn to_dynamic(self) -> Dynamic
    where
        Self: 'static + Sized,
    {
        self.to_shared().to_dynamic()
    }
}

/// Runtime generic shared device.
pub type Dynamic = Shared<dyn Device>;

impl<D: Device + 'static> Shared<D> {
    /// Converts a [`Shared`] into a [`Dynamic`] device.
    #[must_use]
    pub fn to_dynamic(self) -> Dynamic {
        self.into()
    }
}

impl<D: Device + 'static> From<Shared<D>> for Dynamic {
    fn from(dev: Shared<D>) -> Self {
        Self(dev.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Ram;

    #[test]
    fn address_read_works() {
        (0..0x100).for_each(|index| assert_eq!(Ram::from(&[0xaau8; 0x100]).read(index), 0xaa));
    }

    #[test]
    fn address_write_works() {
        let mut dev = Ram::from(&[0u8; 0x100]);
        (0..0x100).for_each(|index| dev.write(index, 0xaa));
        (0..0x100).for_each(|index| assert_eq!(Ram::from(&[0xaau8; 0x100]).read(index), 0xaa));
    }

    #[test]
    fn device_contains_works() {
        (0..0x100).for_each(|index| assert!(Device::contains(&Ram::from(&[0xaau8; 0x100]), index)));
    }
}
