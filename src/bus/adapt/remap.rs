use crate::blk::Block;
use crate::dev::{Device, SharedDevice};

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
pub struct Remap {
    offset: isize,
    dev: SharedDevice,
}

impl Remap {
    /// Constructs a new `Remap`.
    pub fn new(offset: isize, dev: SharedDevice) -> Self {
        Self { offset, dev }
    }
}

impl Block for Remap {
    fn reset(&mut self) {
        self.dev.borrow_mut().reset();
    }
}

impl Device for Remap {
    fn contains(&self, index: usize) -> bool {
        self.dev.borrow().contains(index)
    }

    fn len(&self) -> usize {
        self.dev.borrow().len()
    }

    fn read(&self, index: usize) -> u8 {
        let index = (index as isize - self.offset) as usize;
        self.dev.borrow().read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        let index = (index as isize - self.offset) as usize;
        self.dev.borrow_mut().write(index, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Ram;

    #[test]
    fn new_works() {
        let ram = Ram::<0x100>::new().to_shared();
        let _ = Remap::new(0x0800, ram);
    }

    #[test]
    fn device_contains_works() {
        let ram = Ram::<0x100>::new().to_shared();
        let remap = Remap::new(0x080, ram);
        (0x00..=0x7f).for_each(|addr| assert!(remap.contains(addr)));
        (0x80..=0xff).for_each(|addr| assert!(remap.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        assert_eq!(Remap::new(0x0, Ram::<0x0>::new().to_shared()).len(), 0);
        assert_eq!(Remap::new(0x8, Ram::<0x1>::new().to_shared()).len(), 0x1);
        assert_eq!(Remap::new(0x80, Ram::<0x10>::new().to_shared()).len(), 0x10);
        assert_eq!(
            Remap::new(0x800, Ram::<0x100>::new().to_shared()).len(),
            0x100
        );
        assert_eq!(
            Remap::new(0x8000, Ram::<0x1000>::new().to_shared()).len(),
            0x1000
        );
        assert_eq!(
            Remap::new(0x80000, Ram::<0x10000>::new().to_shared()).len(),
            0x10000
        );
    }

    #[test]
    fn device_read_works() {
        let ram = Ram::<0x100>::from(&[0xaa; 0x100]).to_shared();
        let remap = Remap::new(0x080, ram);
        (0x080..0x180).for_each(|addr| {
            assert_eq!(remap.read(addr), 0xaa);
        });
    }

    #[test]
    fn device_write_works() {
        let ram = Ram::<0x100>::new().to_shared();
        let mut remap = Remap::new(0x080, ram.clone());
        (0x080..0x100).for_each(|addr| {
            remap.write(addr, 0xaa);
        });
        (0x000..0x080).for_each(|addr| {
            assert_eq!(ram.borrow().read(addr), 0xaa);
        });
        (0x080..0x100).for_each(|addr| {
            assert_eq!(ram.borrow().read(addr), 0x00);
        });
    }
}
