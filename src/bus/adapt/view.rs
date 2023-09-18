use std::fmt::Debug;
use std::marker::PhantomData;

use crate::arch::{Address, Value};
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
    phantom: PhantomData<(Idx, V)>,
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
        let index = index + *self.range.start();
        if self.range.contains(&index) {
            self.dev.read(index)
        } else {
            panic!("`<View as Device>::read()`: index out of bounds");
        }
    }

    fn write(&mut self, index: Idx, value: V) {
        let index = index + *self.range.start();
        if self.range.contains(&index) {
            self.dev.write(index, value);
        } else {
            panic!("`<View as Device>::write()`: index out of bounds");
        }
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
        (0..0x80).for_each(|index| {
            assert_eq!(view.read(index), 0xaa);
        });
    }

    #[test]
    fn address_write_works() {
        let ram: Dynamic<usize, u8> = Ram::<u8, 0x100>::new().to_dynamic();
        let mut view = View::new(0x40..=0xbf, ram.clone());
        (0x000..0x080).for_each(|index| {
            view.write(index, 0xaa);
        });
        (0x000..0x040).for_each(|index| {
            assert_eq!(ram.read(index), 0x00);
        });
        (0x040..0x0c0).for_each(|index| {
            assert_eq!(ram.read(index), 0xaa);
        });
        (0x0c0..0x100).for_each(|index| {
            assert_eq!(ram.read(index), 0x00);
        });
    }
}
