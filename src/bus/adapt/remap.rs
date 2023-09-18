#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::marker::PhantomData;

use crate::arch::{Address, Value};
use crate::blk::Block;
use crate::dev::Device;

/// Address remap.
///
/// # Usage
///
/// The `Remap` adapter shifts the [`Device`]'s effective address space by the
/// provided offset.
///
/// In conjunction with [`View`](super::View), devices can be partially or
/// completely mapped into another address space as desired.
#[derive(Debug)]
pub struct Remap<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    off: Idx,
    dev: T,
    phantom: PhantomData<(Idx, V)>,
}

impl<T, Idx, V> Remap<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    /// Constructs a new `Remap`.
    #[must_use]
    pub fn new(off: Idx, dev: T) -> Self {
        Self {
            off,
            dev,
            phantom: PhantomData,
        }
    }
}

impl<T, Idx, V> Address<Idx, V> for Remap<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    fn read(&self, index: Idx) -> V {
        let index = index - self.off;
        self.dev.read(index)
    }

    fn write(&mut self, index: Idx, value: V) {
        let index = index - self.off;
        self.dev.write(index, value);
    }
}

impl<T, Idx, V> Block for Remap<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    fn reset(&mut self) {
        self.dev.reset();
    }
}

impl<T, Idx, V> Device<Idx, V> for Remap<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Ram;

    #[test]
    fn new_works() {
        let ram = Ram::<u8, 0x100>::new();
        let _: Remap<_, usize, u8> = Remap::new(0x0800, ram);
    }

    #[test]
    fn address_read_works() {
        let ram = Ram::from(&[0xaa; 0x100]);
        let remap: Remap<_, usize, u8> = Remap::new(0x080, ram);
        (0x080..0x180).for_each(|index| {
            assert_eq!(remap.read(index), 0xaa);
        });
    }

    #[test]
    fn address_write_works() {
        let ram = Ram::<u8, 0x100>::new().to_dynamic();
        let mut remap: Remap<_, usize, u8> = Remap::new(0x080, ram.clone());
        (0x080..0x100).for_each(|index: usize| {
            remap.write(index, 0xaa);
        });
        (0x000..0x080).for_each(|index: usize| {
            assert_eq!(ram.read(index), 0xaa);
        });
        (0x080..0x100).for_each(|index: usize| {
            assert_eq!(ram.read(index), 0x00);
        });
    }
}
