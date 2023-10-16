use std::cell::RefCell;

use crate::arch::{TryAddress, Value};
use crate::bus::{Mux, Range};
use crate::dev::{Device, Dynamic};
use crate::{Address, Block, Machine};

/// Buffered device.
///
/// Uses [`Wire`]s to model registered I/O for device requests.
#[derive(Debug)]
pub struct Wired<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    pub inner: T,
    index: RefCell<Wire<Idx>>,
    value: RefCell<Wire<V>>,
}

impl<T, Idx, V> Address<Idx, V> for Wired<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
    fn read(&self, index: Idx) -> V {
        let index = self.index.borrow().get().unwrap_or(index);
        let value = self
            .value
            .borrow()
            .get()
            .unwrap_or_else(|| self.inner.read(index));
        self.index.borrow_mut().acquire(index);
        self.value.borrow_mut().acquire(value);
        value
    }

    fn write(&mut self, index: Idx, value: V) {
        self.inner.write(
            self.index.borrow().get().unwrap_or(index),
            self.value.borrow().get().unwrap_or(value),
        );
    }
}

impl<T, Idx, V> TryAddress<Idx, V> for Wired<T, Idx, V>
where
    T: Device<Idx, V> + TryAddress<Idx, V>,
    Idx: Value,
    V: Value,
{
    type Error = <T as TryAddress<Idx, V>>::Error;

    fn try_read(&self, index: Idx) -> Result<V, Self::Error> {
        let index = self.index.borrow().get().unwrap_or(index);
        let value = self
            .value
            .borrow()
            .get()
            .map_or_else(|| self.inner.try_read(index), |value| Ok(value))?;
        self.index.borrow_mut().acquire(index);
        self.value.borrow_mut().acquire(value);
        Ok(value)
    }

    fn try_write(&mut self, index: Idx, value: V) -> Result<(), Self::Error> {
        self.inner.try_write(
            self.index.borrow().get().unwrap_or(index),
            self.value.borrow().get().unwrap_or(value),
        )
    }
}

impl<T, Idx, V> Block for Wired<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
}

impl<T, Idx, V> Device<Idx, V> for Wired<T, Idx, V>
where
    T: Device<Idx, V>,
    Idx: Value,
    V: Value,
{
}

impl<T, Idx, V> Machine for Wired<T, Idx, V>
where
    T: Device<Idx, V> + Machine,
    Idx: Value,
    V: Value,
{
    fn enabled(&self) -> bool {
        self.inner.enabled()
    }

    fn cycle(&mut self) {
        self.index.get_mut().release();
        self.value.get_mut().release();
        self.inner.cycle();
    }
}

impl<T, Idx, V> Mux<Idx, V> for Wired<T, Idx, V>
where
    T: Device<Idx, V> + Mux<Idx, V>,
    Idx: Value,
    V: Value,
{
    fn get(&self, index: Idx) -> Option<Dynamic<Idx, V>> {
        self.inner.get(index)
    }

    fn map(&mut self, range: Range<Idx>, dev: Dynamic<Idx, V>) {
        self.inner.map(range, dev);
    }

    fn unmap(&mut self, dev: &Dynamic<Idx, V>) -> Option<Dynamic<Idx, V>> {
        self.inner.unmap(dev)
    }
}

/// Tri-state buffered wire.
#[derive(Copy, Clone, Debug, Default)]
pub enum Wire<V>
where
    V: Value,
{
    /// High impedance wire state.
    ///
    /// No value currently asserted.
    #[default]
    Impedant,
    /// Active wire state.
    ///
    /// Holds value being asserted.
    Active(V),
}

impl<V> Wire<V>
where
    V: Value,
{
    /// Acquires the wire until it is released.
    ///
    /// This is intended to model a wire that is being driven by a device.
    pub fn acquire(&mut self, value: V) {
        *self = Self::Active(value);
    }

    /// Releases the wire.
    ///
    /// This is intended to model a wire that is not currently being driven by
    /// any devices.
    /// being asserted on the wire.
    pub fn release(&mut self) {
        *self = Self::Impedant;
    }

    /// Gets the value, if any, asserted on the wire.
    pub fn get(&self) -> Option<V> {
        match self {
            Wire::Impedant => None,
            Wire::Active(value) => Some(*value),
        }
    }
}
