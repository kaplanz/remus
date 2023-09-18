use std::cell::RefCell;

use crate::arch::Value;
use crate::dev::Device;
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
    inner: T,
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
