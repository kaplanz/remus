//! Generic memory mapped device.
//!
//! # Usage
//!
//! The [`Device`] trait is useful in combination with [`Bus`](crate::bus::Bus).
//! Together, they can be used to emulate the behaviour of [memory-mapped I/O].
//!
//! [memory-mapped I/O]: https://en.wikipedia.org/wiki/Memory-mapped_I/O

use crate::arch::{Address, Value};
use crate::blk::Block;
use crate::share::Shared;

mod null;
mod random;

pub use self::null::Null;
pub use self::random::Random;

/// Memory-mapped I/O device.
pub trait Device<Idx, V>: Address<Idx, V> + Block
where
    Idx: Value,
    V: Value,
{
    /// Constructs a [`Shared`] device from `self`.
    fn to_shared(self) -> Shared<Self>
    where
        Self: 'static + Sized,
    {
        self.into()
    }

    /// Constructs a [`Dynamic`] device from `self`.
    fn to_dynamic(self) -> Dynamic<Idx, V>
    where
        Self: 'static + Sized,
    {
        self.to_shared().into()
    }
}

/// Runtime generic shared device.
pub type Dynamic<Idx, V> = Shared<dyn Device<Idx, V>>;

impl<T, Idx, V> From<Shared<T>> for Dynamic<Idx, V>
where
    T: Device<Idx, V> + 'static,
    Idx: Value,
    V: Value,
{
    fn from(dev: Shared<T>) -> Self {
        Self(dev.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Ram;

    #[test]
    fn address_read_works() {
        (0..0x100)
            .for_each(|index: usize| assert_eq!(Ram::from(&[0xaau8; 0x100]).read(index), 0xaa));
    }

    #[test]
    fn address_write_works() {
        let mut dev = Ram::from(&[0u8; 0x100]);
        (0..0x100).for_each(|index: usize| dev.write(index, 0xaa));
        (0..0x100)
            .for_each(|index: usize| assert_eq!(Ram::from(&[0xaau8; 0x100]).read(index), 0xaa));
    }
}
