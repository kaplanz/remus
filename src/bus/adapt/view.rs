use std::fmt::Debug;
use std::marker::PhantomData;

use crate::arch::{Address, TryAddress, Value};
use crate::blk::Block;
use crate::bus::Range;
use crate::dev::Device;

/// Partial address view.
///
/// # Usage
///
/// The `View` device adapter allows access to a slice of the underlying
/// [`Device`], while remapping the starting address to zero.
///
/// In conjunction with [`Remap`](super::Remap), devices can be partially or
/// completely mapped into another address space as desired.
#[derive(Debug)]
pub struct View<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    range: Range<Idx>,
    dev: T,
    phantom: PhantomData<V>,
}

impl<T, Idx, V> View<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    /// Constructs a new `View`.
    pub fn new(range: Range<Idx>, dev: T) -> Self {
        Self {
            dev,
            range,
            phantom: PhantomData,
        }
    }
}

impl<T, Idx, V> Address<Idx, V> for View<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    fn read(&self, index: Idx) -> V {
        self.try_read(index)
            .expect("`<View as Address>::read`: index out of bounds: {index}")
    }

    fn write(&mut self, index: Idx, value: V) {
        self.try_write(index, value)
            .expect("`<View as Address>::write`: index out of bounds: {index}");
    }
}

impl<T, Idx, V> TryAddress<Idx, V> for View<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    fn try_read(&self, index: Idx) -> Option<V> {
        let index = index + *self.range.start();
        self.range.contains(&index).then(|| self.dev.read(index))
    }

    fn try_write(&mut self, index: Idx, value: V) -> Option<()> {
        let index = index + *self.range.start();
        self.range
            .contains(&index)
            .then(|| self.dev.write(index, value))
    }
}

impl<T, Idx, V> Block for View<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    fn reset(&mut self) {
        self.dev.reset();
    }
}

impl<T, Idx, V> Device<Idx, V> for View<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::Dynamic;
    use crate::mem::Ram;

    #[test]
    fn new_works() {
        let ram = Ram::<u8, 0x100>::new();
        let _: View<_, u8, _> = View::new(0x40..=0xbf, ram);
    }

    #[test]
    fn address_read_works() {
        let ram: Dynamic<usize, u8> = Ram::from(&[0xaa; 0x100]).to_dynamic();
        let view = View::new(0x40..=0xbf, ram);
        (0x00..=0x7f).for_each(|index| {
            assert_eq!(view.read(index), 0xaa);
        });
        (0x80..=0xff).for_each(|index| {
            assert_eq!(view.try_read(index), None);
        });
    }

    #[test]
    fn address_write_works() {
        let ram: Dynamic<usize, u8> = Ram::<u8, 0x100>::new().to_dynamic();
        let mut view = View::new(0x40..=0xbf, ram.clone());
        (0x00..=0x7f).for_each(|index| {
            view.write(index, 0xaa);
        });
        (0x80..=0xff).for_each(|index| {
            assert_eq!(view.try_write(index, 0xaa), None);
        });
        (0x00..=0x3f).for_each(|index| {
            assert_eq!(ram.read(index), 0x00);
        });
        (0x40..=0xbf).for_each(|index| {
            assert_eq!(ram.read(index), 0xaa);
        });
        (0xc0..=0xff).for_each(|index| {
            assert_eq!(ram.read(index), 0x00);
        });
    }
}
