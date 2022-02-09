use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};

use super::DynDevice;
use crate::blk::Block;
use crate::dev::Device;

/// View device adapter.
///
/// # Usage
///
/// The [`View`] device adapter allows access to a slice of the underlying
/// device, while remapping the starting address to zero.
///
/// In conjunction with [`Remap`](super::Remap), devices can be partially or
/// completely mapped into another address space as desired.
#[derive(Debug)]
pub struct View<R: Debug + RangeBounds<usize>> {
    dev: DynDevice,
    range: R,
}

impl<R: Debug + RangeBounds<usize>> View<R> {
    pub fn new(dev: DynDevice, range: R) -> Self {
        Self { dev, range }
    }
}

impl<R: Debug + RangeBounds<usize>> Block for View<R> {}

impl<R: Debug + RangeBounds<usize>> Device for View<R> {
    fn contains(&self, index: usize) -> bool {
        self.range.contains(&index)
    }

    fn read(&self, index: usize) -> u8 {
        let index = index
            + match self.range.start_bound() {
                Bound::Included(start) => *start,
                Bound::Excluded(start) => *start + 1,
                Bound::Unbounded => 0,
            };
        if self.range.contains(&index) {
            self.dev.borrow().read(index)
        } else {
            panic!("`<View as Device>::read()`: index out of bounds");
        }
    }

    fn write(&mut self, index: usize, value: u8) {
        let index = index
            + match self.range.start_bound() {
                Bound::Included(start) => *start,
                Bound::Excluded(start) => *start + 1,
                Bound::Unbounded => 0,
            };
        if self.range.contains(&index) {
            self.dev.borrow_mut().write(index, value);
        } else {
            panic!("`<View as Device>::write()`: index out of bounds");
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
    fn device_contains_works() {
        // Exclusive bound
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
        let view = View::new(ram, 0x40..0xc0);
        (0x00..=0x3f).for_each(|addr| assert!(!view.contains(addr)));
        (0x40..=0xbf).for_each(|addr| assert!(view.contains(addr)));
        (0xc0..=0xff).for_each(|addr| assert!(!view.contains(addr)));

        // Inclusive bound
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
        let view = View::new(ram, 0x40..=0xbf);
        (0x00..=0x3f).for_each(|addr| assert!(!view.contains(addr)));
        (0x40..=0xbf).for_each(|addr| assert!(view.contains(addr)));
        (0xc0..=0xff).for_each(|addr| assert!(!view.contains(addr)));
    }

    #[test]
    fn device_read_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::from(&[0xaa; 0x100])));
        let view = View::new(ram, 0x40..0xc0);
        (0..0x80).for_each(|addr| {
            assert_eq!(view.read(addr), 0xaa);
        });
    }

    #[test]
    fn device_write_works() {
        let ram = Rc::new(RefCell::new(Ram::<0x100>::new()));
        let mut view = View::new(ram.clone(), 0x40..0xc0);
        (0x000..0x080).for_each(|addr| {
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
