use std::ops::{Deref, DerefMut};

use crate::arch::{TryAddress, Value};
use crate::blk::Block;
use crate::bus::{self, Bus};
use crate::dev::Device;
use crate::Address;

/// Bus mask.
///
/// # Usage
///
/// The `Mask` adapter...
#[derive(Debug, Default)]
pub struct Mask<Idx, V>(Vec<Layer<Idx, V>>)
where
    Idx: Value,
    V: Value;

impl<Idx, V> Mask<Idx, V>
where
    Idx: Value,
    V: Value,
{
    /// Constructs a new `Mask`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Idx, V> Address<Idx, V> for Mask<Idx, V>
where
    Idx: Value,
    V: Value,
{
    fn read(&self, index: Idx) -> V {
        self.try_read(index).unwrap()
    }

    fn write(&mut self, index: Idx, value: V) {
        self.try_write(index, value).unwrap();
    }
}

impl<Idx, V> TryAddress<Idx, V> for Mask<Idx, V>
where
    Idx: Value,
    V: Value,
{
    type Error = Error<Idx>;

    fn try_read(&self, index: Idx) -> Result<V, Self::Error> {
        self.0
            .iter()
            .filter(|layer| !layer.skip)
            .find_map(|layer| layer.bus.try_read(index).ok())
            .ok_or(Error::Unmapped(index))
    }

    fn try_write(&mut self, index: Idx, value: V) -> Result<(), Self::Error> {
        self.0
            .iter_mut()
            .filter(|layer| !layer.skip)
            .find_map(|layer| layer.bus.try_write(index, value).ok())
            .ok_or(Error::Unmapped(index))
    }
}

impl<Idx, V> Block for Mask<Idx, V>
where
    Idx: Value,
    V: Value,
{
}

impl<Idx, V> Deref for Mask<Idx, V>
where
    Idx: Value,
    V: Value,
{
    type Target = Vec<Layer<Idx, V>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Idx, V> DerefMut for Mask<Idx, V>
where
    Idx: Value,
    V: Value,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Idx, V> Device<Idx, V> for Mask<Idx, V>
where
    Idx: Value,
    V: Value,
{
}

#[derive(Debug)]
pub struct Layer<Idx, V>
where
    Idx: Value,
    V: Value,
{
    pub bus: Bus<Idx, V>,
    pub skip: bool,
}

impl<Idx, V> Layer<Idx, V>
where
    Idx: Value,
    V: Value,
{
    #[must_use]
    pub fn new(bus: Bus<Idx, V>) -> Self {
        Self { bus, skip: false }
    }
}

/// A type specifying general categories of [`Mask`] error.
pub type Error<Idx> = bus::Error<Idx>;
