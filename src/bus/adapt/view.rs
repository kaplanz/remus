use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};

use crate::blk::Block;
use crate::dev::{Device, SharedDevice};

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
pub struct View<R>
where
    R: Debug + RangeBounds<usize>,
{
    dev: SharedDevice,
    range: R,
}

impl<R> View<R>
where
    R: Debug + RangeBounds<usize>,
{
    /// Constructs a new `View`.
    pub fn new(dev: SharedDevice, range: R) -> Self {
        Self { dev, range }
    }

    fn translate(&self, index: usize) -> usize {
        index
            + match self.range.start_bound() {
                Bound::Included(&start) => start,
                Bound::Excluded(&start) => start.saturating_add(1),
                Bound::Unbounded => 0,
            }
    }
}

impl<R> Block for View<R>
where
    R: Debug + RangeBounds<usize>,
{
    fn reset(&mut self) {
        self.dev.borrow_mut().reset();
    }
}

impl<R> Device for View<R>
where
    R: Debug + RangeBounds<usize>,
{
    fn contains(&self, index: usize) -> bool {
        let index = self.translate(index);
        self.range.contains(&index)
    }

    fn len(&self) -> usize {
        let start = match self.range.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let end = match self.range.end_bound() {
            Bound::Included(&end) => end.saturating_add(1),
            Bound::Excluded(&end) => end,
            Bound::Unbounded => usize::MAX,
        };
        end.saturating_sub(start)
    }

    fn read(&self, index: usize) -> u8 {
        let index = self.translate(index);
        if self.range.contains(&index) {
            self.dev.borrow().read(index)
        } else {
            panic!("`<View as Device>::read()`: index out of bounds");
        }
    }

    fn write(&mut self, index: usize, value: u8) {
        let index = self.translate(index);
        if self.range.contains(&index) {
            self.dev.borrow_mut().write(index, value);
        } else {
            panic!("`<View as Device>::write()`: index out of bounds");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Ram;

    #[test]
    fn new_works() {
        let ram = Ram::<0x100>::new().to_shared();
        let _ = View::new(ram, 0x40..0xc0);
    }

    #[test]
    fn device_contains_works() {
        // Exclusive bound
        let ram = Ram::<0x100>::new().to_shared();
        let view = View::new(ram, 0x40..0xc0);
        (0x00..=0x7f).for_each(|addr| assert!(view.contains(addr)));
        (0x80..=0xff).for_each(|addr| assert!(!view.contains(addr)));

        // Inclusive bound
        let ram = Ram::<0x100>::new().to_shared();
        let view = View::new(ram, 0x40..=0xbf);
        (0x00..=0x7f).for_each(|addr| assert!(view.contains(addr)));
        (0x80..=0xff).for_each(|addr| assert!(!view.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        let ram = Ram::<0x10000>::new().to_shared();
        assert_eq!(View::new(ram.clone(), 0..0x0).len(), 0x0);
        assert_eq!(View::new(ram.clone(), 0..=0x0).len(), 0x1);
        assert_eq!(View::new(ram.clone(), 0..0x10).len(), 0x10);
        assert_eq!(View::new(ram.clone(), 0..=0xff).len(), 0x100);
        assert_eq!(View::new(ram.clone(), 0..0x1000).len(), 0x1000);
        assert_eq!(View::new(ram, 0..=0xffff).len(), 0x10000);
    }

    #[test]
    fn device_read_works() {
        let ram = Ram::<0x100>::from(&[0xaa; 0x100]).to_shared();
        let view = View::new(ram, 0x40..0xc0);
        (0..0x80).for_each(|addr| {
            assert_eq!(view.read(addr), 0xaa);
        });
    }

    #[test]
    fn device_write_works() {
        let ram = Ram::<0x100>::new().to_shared();
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
