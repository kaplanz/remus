use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crate::arch::{Address, Cell, Location};
use crate::blk::{Block, Linked};
use crate::bus::Bus;
use crate::dev::Device;
use crate::fsm::Machine;
use crate::pcb::Board;

/// Heap-allocated multi-access resource.
#[derive(Debug, Default, Eq)]
pub struct Shared<T: ?Sized>(pub(crate) Inner<T>);

impl<T> Shared<T>
where
    T: 'static,
{
    /// Creates a new [`Shared`] resource.
    pub fn new(dev: T) -> Self {
        Self(Rc::new(RefCell::new(dev)))
    }

    /// Gets a reference to the underlying inner smart pointer.
    #[must_use]
    pub fn inner(&self) -> &Inner<T> {
        &self.0
    }

    /// Mutable gets a reference to the underlying inner smart pointer.
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut Inner<T> {
        &mut self.0
    }
}

impl<T> Shared<T>
where
    T: ?Sized,
{
    #[must_use]
    pub fn borrow(&self) -> Ref<T> {
        self.0.borrow()
    }

    #[must_use]
    pub fn borrow_mut(&self) -> RefMut<T> {
        self.0.borrow_mut()
    }
}

impl<T> Address<u8> for Shared<T>
where
    T: Address<u8> + ?Sized,
{
    fn read(&self, index: usize) -> u8 {
        self.0.read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        self.0.write(index, value);
    }
}

impl<T> Block for Shared<T>
where
    T: Block + ?Sized,
{
    fn reset(&mut self) {
        self.0.reset();
    }
}

impl<T> Board for Shared<T>
where
    T: Board + ?Sized,
{
    fn connect(&self, bus: &mut Bus) {
        self.0.connect(bus);
    }

    fn disconnect(&self, bus: &mut Bus) {
        self.0.disconnect(bus);
    }
}

impl<T, V> Cell<V> for Shared<T>
where
    T: Cell<V> + ?Sized,
    V: Copy + Default,
{
    fn load(&self) -> V {
        self.0.load()
    }

    fn store(&mut self, value: V) {
        self.0.store(value);
    }
}

impl<T> Clone for Shared<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Device for Shared<T>
where
    T: Device + ?Sized,
{
    fn contains(&self, index: usize) -> bool {
        self.0.contains(index)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> From<T> for Shared<T>
where
    T: 'static,
{
    fn from(dev: T) -> Self {
        Self::new(dev)
    }
}

impl<T> PartialEq for Shared<T>
where
    T: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T, B> Linked<B> for Shared<T>
where
    T: Linked<B> + ?Sized,
    B: Block,
{
    fn mine(&self) -> Shared<B> {
        self.0.mine()
    }

    fn link(&mut self, it: Shared<B>) {
        self.0.link(it);
    }
}

impl<T, V> Location<V> for Shared<T>
where
    T: Location<V> + ?Sized,
    V: Copy + Default,
{
    type Register = T::Register;

    fn load(&self, reg: Self::Register) -> V {
        self.0.load(reg)
    }

    fn store(&mut self, reg: Self::Register, value: V) {
        self.0.store(reg, value);
    }
}

impl<T> Machine for Shared<T>
where
    T: Machine + ?Sized,
{
    fn enabled(&self) -> bool {
        self.0.enabled()
    }

    fn cycle(&mut self) {
        self.0.cycle();
    }
}

/// Internal shared reference type.
pub(crate) type Inner<T> = Rc<RefCell<T>>;

impl<T> Address<u8> for Inner<T>
where
    T: Address<u8> + ?Sized,
{
    fn read(&self, index: usize) -> u8 {
        self.borrow().read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        self.borrow_mut().write(index, value);
    }
}

impl<T> Block for Inner<T>
where
    T: Block + ?Sized,
{
    fn reset(&mut self) {
        self.borrow_mut().reset();
    }
}

impl<T> Board for Inner<T>
where
    T: Board + ?Sized,
{
    fn connect(&self, bus: &mut Bus) {
        self.borrow().connect(bus);
    }

    fn disconnect(&self, bus: &mut Bus) {
        self.borrow_mut().disconnect(bus);
    }
}

impl<T, V> Cell<V> for Inner<T>
where
    T: Cell<V> + ?Sized,
    V: Copy + Default,
{
    fn load(&self) -> V {
        self.borrow().load()
    }

    fn store(&mut self, value: V) {
        self.borrow_mut().store(value);
    }
}

impl<T> Device for Inner<T>
where
    T: Device + ?Sized,
{
    fn contains(&self, index: usize) -> bool {
        self.borrow().contains(index)
    }

    fn len(&self) -> usize {
        self.borrow().len()
    }
}

impl<T, B> Linked<B> for Inner<T>
where
    T: Linked<B> + ?Sized,
    B: Block,
{
    fn mine(&self) -> Shared<B> {
        self.borrow().mine()
    }

    fn link(&mut self, it: Shared<B>) {
        self.borrow_mut().link(it);
    }
}

impl<T, V> Location<V> for Inner<T>
where
    T: Location<V> + ?Sized,
    V: Copy + Default,
{
    type Register = T::Register;

    fn load(&self, reg: Self::Register) -> V {
        self.borrow().load(reg)
    }

    fn store(&mut self, reg: Self::Register, value: V) {
        self.borrow_mut().store(reg, value);
    }
}

impl<T> Machine for Inner<T>
where
    T: Machine + ?Sized,
{
    fn enabled(&self) -> bool {
        self.borrow().enabled()
    }

    fn cycle(&mut self) {
        self.borrow_mut().cycle();
    }
}
