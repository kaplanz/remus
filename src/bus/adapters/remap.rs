use super::DynDevice;
use crate::blk::Block;
use crate::dev::Device;

/// Remap device adapter.
///
/// # Usage
///
/// The [`Remap`] device adapter shifts the effective address space by the
/// provided offset.
///
/// In conjunction with [`View`](super::View), devices can be partially or
/// completely mapped into another address space as desired.
#[derive(Debug)]
pub struct Remap {
    offset: isize,
    dev: DynDevice,
}

impl Remap {
    pub fn new(offset: isize, dev: DynDevice) -> Self {
        Self { offset, dev }
    }
}

impl Block for Remap {}

impl Device for Remap {
    fn contains(&self, index: usize) -> bool {
        self.dev.borrow().contains(index)
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
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::mem::Ram;

    #[test]
    fn new_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
        let _ = Remap::new(0x0800, ram);
    }

    #[test]
    fn device_contains_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
        let remap = Remap::new(0x080, ram);
        (0x00..=0x7f).for_each(|addr| assert!(remap.contains(addr)));
        (0x80..=0xff).for_each(|addr| assert!(remap.contains(addr)));
    }

    #[test]
    fn device_read_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::from(&[0xaa; 0x100])));
        let remap = Remap::new(0x080, ram);
        (0x080..0x180).for_each(|addr| {
            assert_eq!(remap.read(addr), 0xaa);
        });
    }

    #[test]
    fn device_write_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
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
