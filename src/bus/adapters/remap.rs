use super::DynDevice;
use crate::dev::Device;

/// Remap device adapter.
///
/// # Usage
///
/// The [`Remap`] device adapter shifts the effective address space by the
/// provided offset.
///
/// In conjunction with [`super::View`], devices can be partially or completely mapped
/// into another address space as desired.
#[derive(Debug)]
pub struct Remap {
    dev: DynDevice,
    offset: isize,
}

impl Remap {
    pub fn new(dev: DynDevice, offset: isize) -> Self {
        Self { dev, offset }
    }
}

impl Device for Remap {
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
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::mem::Ram;

    #[test]
    fn new_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
        let _ = Remap::new(ram, 0x080);
    }

    #[test]
    fn device_len_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
        let remap = Remap::new(ram, 0x080);
        assert_eq!(remap.len(), 0x100);
    }

    #[test]
    fn device_read_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::from(&[0xaa; 0x100])));
        let remap = Remap::new(ram, 0x080);
        (0x080..0x180).for_each(|addr| {
            assert_eq!(remap.read(addr), 0xaa);
        });
    }

    #[test]
    fn device_write_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
        let mut remap = Remap::new(ram.clone(), 0x080);
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
