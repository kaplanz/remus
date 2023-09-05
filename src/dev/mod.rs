//! Generic memory mapped device.
//!
//! # Usage
//!
//! The [`Device`] trait is useful in combination with [`Bus`](crate::bus::Bus).
//! Together, they can be used to emulate the behaviour of [memory-mapped I/O].
//!
//! [memory-mapped I/O]: https://en.wikipedia.org/wiki/Memory-mapped_I/O

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::arc::{Address, Cell};
use crate::blk::Block;

mod null;
mod random;

pub use self::null::Null;
pub use self::random::Random;

/// Memory-mapped I/O device.
pub trait Device: Address<u8> + Block {
    /// Checks if the device contains the provided `index` within its
    /// address space.
    fn contains(&self, index: usize) -> bool;

    /// Returns the length of the device, in bytes.
    fn len(&self) -> usize;

    /// Checks if the length of the device is zero.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Constructs a [`Shared`] device from `self`.
    fn to_shared(self) -> Shared<Self>
    where
        Self: 'static + Sized,
    {
        self.into()
    }

    /// Constructs a [`Dynamic`] device from `self`.
    fn to_dynamic(self) -> Dynamic
    where
        Self: 'static + Sized,
    {
        self.to_shared().to_dynamic()
    }
}

type Inner<T> = Rc<RefCell<T>>;

impl<D: Address<u8> + ?Sized> Address<u8> for Inner<D> {
    fn read(&self, index: usize) -> u8 {
        self.borrow().read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        self.borrow_mut().write(index, value);
    }
}

impl<D: Block + ?Sized> Block for Inner<D> {
    fn reset(&mut self) {
        self.borrow_mut().reset();
    }
}

impl<D, V> Cell<V> for Inner<D>
where
    D: Cell<V> + ?Sized,
    V: Copy + Default,
{
    fn load(&self) -> V {
        self.borrow().load()
    }

    fn store(&mut self, value: V) {
        self.borrow_mut().store(value);
    }
}

impl<D: Device + ?Sized> Device for Inner<D> {
    fn contains(&self, index: usize) -> bool {
        self.borrow().contains(index)
    }

    fn len(&self) -> usize {
        self.borrow().len()
    }
}

/// Runtime generic shared device.
pub type Dynamic = Shared<dyn Device>;

/// Heap-allocated multi-access device.
#[derive(Debug, Default)]
pub struct Shared<D: Device + ?Sized>(Inner<D>);

impl<D: Device + 'static> Shared<D> {
    /// Creates a new [`Shared`] [`Device`].
    pub fn new(dev: D) -> Self {
        Self(Rc::new(RefCell::new(dev)))
    }

    /// Gets a reference to the underlying inner smart pointer.
    #[must_use]
    pub fn inner(&self) -> &Inner<D> {
        &self.0
    }

    /// Mutably gets a reference to the underlying inner smart pointer.
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut Inner<D> {
        &mut self.0
    }

    /// Converts a [`Shared`] into a [`Dynamic`] device.
    #[must_use]
    pub fn to_dynamic(self) -> Dynamic {
        self.into()
    }
}

impl<D: Device + ?Sized> Shared<D> {
    #[must_use]
    pub fn borrow(&self) -> Ref<D> {
        self.0.borrow()
    }

    #[must_use]
    pub fn borrow_mut(&self) -> RefMut<D> {
        self.0.borrow_mut()
    }
}

impl<D: Device + ?Sized> Address<u8> for Shared<D> {
    fn read(&self, index: usize) -> u8 {
        self.0.read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        self.0.write(index, value);
    }
}

impl<D: Device + ?Sized> Block for Shared<D> {
    fn reset(&mut self) {
        self.0.reset();
    }
}

impl<D, V> Cell<V> for Shared<D>
where
    D: Cell<V> + Device + ?Sized,
    V: Copy + Default,
{
    fn load(&self) -> V {
        self.0.load()
    }

    fn store(&mut self, value: V) {
        self.0.store(value);
    }
}

impl<D: Device + ?Sized> Clone for Shared<D> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<D: Device + ?Sized> Device for Shared<D> {
    fn contains(&self, index: usize) -> bool {
        self.0.contains(index)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<D: Device + 'static> From<D> for Shared<D> {
    fn from(dev: D) -> Self {
        Self::new(dev)
    }
}

impl<D: Device + 'static> From<Shared<D>> for Dynamic {
    fn from(dev: Shared<D>) -> Self {
        Self(dev.0.clone())
    }
}

impl<D: Device + ?Sized> PartialEq for Shared<D> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Ram;

    #[test]
    fn address_read_works() {
        (0..0x100).for_each(|index| assert_eq!(Ram::from(&[0xaau8; 0x100]).read(index), 0xaa));
    }

    #[test]
    fn address_write_works() {
        let mut dev = Ram::from(&[0u8; 0x100]);
        (0..0x100).for_each(|index| dev.write(index, 0xaa));
        (0..0x100).for_each(|index| assert_eq!(Ram::from(&[0xaau8; 0x100]).read(index), 0xaa));
    }

    #[test]
    fn device_contains_works() {
        (0..0x100).for_each(|index| assert!(Device::contains(&Ram::from(&[0xaau8; 0x100]), index)));
    }
}
