#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use crate::arch::Address;
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
pub struct Remap<D: Device> {
    offset: isize,
    dev: D,
}

impl<D: Device> Remap<D> {
    /// Constructs a new `Remap`.
    #[must_use]
    pub fn new(offset: isize, dev: D) -> Self {
        Self { offset, dev }
    }
}

impl<D: Device> Address<u8> for Remap<D> {
    fn read(&self, index: usize) -> u8 {
        let index = (index as isize - self.offset) as usize;
        self.dev.read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        let index = (index as isize - self.offset) as usize;
        self.dev.write(index, value);
    }
}

impl<D: Device> Block for Remap<D> {
    fn reset(&mut self) {
        self.dev.reset();
    }
}

impl<D: Device> Device for Remap<D> {
    fn contains(&self, index: usize) -> bool {
        self.dev.contains(index)
    }

    fn len(&self) -> usize {
        self.dev.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Ram;

    #[test]
    fn new_works() {
        let ram = Ram::<0x100>::new();
        let _ = Remap::new(0x0800, ram);
    }

    #[test]
    fn address_read_works() {
        let ram = Ram::<0x100>::from(&[0xaa; 0x100]);
        let remap = Remap::new(0x080, ram);
        (0x080..0x180).for_each(|addr| {
            assert_eq!(remap.read(addr), 0xaa);
        });
    }

    #[test]
    fn address_write_works() {
        let ram = Ram::<0x100>::new().to_shared();
        let mut remap = Remap::new(0x080, ram.clone());
        (0x080..0x100).for_each(|addr| {
            remap.write(addr, 0xaa);
        });
        (0x000..0x080).for_each(|addr| {
            assert_eq!(ram.read(addr), 0xaa);
        });
        (0x080..0x100).for_each(|addr| {
            assert_eq!(ram.read(addr), 0x00);
        });
    }

    #[test]
    fn device_contains_works() {
        let ram = Ram::<0x100>::new();
        let remap = Remap::new(0x080, ram);
        (0x00..=0x7f).for_each(|addr| assert!(remap.contains(addr)));
        (0x80..=0xff).for_each(|addr| assert!(remap.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        assert_eq!(Remap::new(0x0, Ram::<0x0>::new()).len(), 0);
        assert_eq!(Remap::new(0x8, Ram::<0x1>::new()).len(), 0x1);
        assert_eq!(Remap::new(0x80, Ram::<0x10>::new()).len(), 0x10);
        assert_eq!(Remap::new(0x800, Ram::<0x100>::new()).len(), 0x100);
        assert_eq!(Remap::new(0x8000, Ram::<0x1000>::new()).len(), 0x1000);
        assert_eq!(Remap::new(0x80000, Ram::<0x10000>::new()).len(), 0x10000);
    }
}
