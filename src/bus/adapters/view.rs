use std::ops::Range;

use super::DynDevice;
use crate::dev::Device;

/// View device adapter.
///
/// # Usage
///
/// The [`View`] device adapter allows access some or all of the underlying
/// device, remapping the starting address to zero.
///
/// In conjunction with [`super::Remap`], devices can be partially or completely mapped
/// into another address space as desired.
#[derive(Debug)]
pub struct View {
    dev: DynDevice,
    range: Range<usize>,
}

impl View {
    pub fn new(dev: DynDevice, range: Range<usize>) -> Self {
        Self { dev, range }
    }
}

impl Device for View {
    fn len(&self) -> usize {
        self.range.end - self.range.start
    }

    fn read(&self, index: usize) -> u8 {
        let index = self.range.start + index;
        if self.range.contains(&index) {
            self.dev.borrow().read(index)
        } else {
            panic!("`<View as Device>::read()`: index out of bounds");
        }
    }

    fn write(&mut self, index: usize, value: u8) {
        let index = self.range.start + index;
        if self.range.contains(&index) {
            self.dev.borrow_mut().write(index, value);
        } else {
            panic!("`<View as Device>::read()`: index out of bounds");
        }
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
        let _ = View::new(ram, 0x40..0xc0);
    }

    #[test]
    fn device_len_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
        let view = View::new(ram, 0x40..0xc0);
        assert_eq!(view.len(), 0x80);
    }

    #[test]
    fn device_read_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::from(&[0xaa; 0x100])));
        let view = View::new(ram, 0x40..0xc0);
        (0..view.len()).for_each(|addr| {
            assert_eq!(view.read(addr), 0xaa);
        });
    }

    #[test]
    fn device_write_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
        let mut view = View::new(ram.clone(), 0x40..0xc0);
        (0..view.len()).for_each(|addr| {
            view.write(addr, 0xaa);
        });
        (0x000..0x040).for_each(|addr| {
            assert_eq!(ram.borrow().read(addr), 0x00);
        });
        (0x040..0x0c0).for_each(|addr| {
            assert_eq!(ram.borrow().read(addr), 0xaa);
        });
        (0x0c0..0x100).for_each(|addr| {
            assert_eq!(ram.borrow().read(addr), 0x00);
        });
    }
}
