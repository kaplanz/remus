use std::marker::PhantomData;

use crate::arch::{TryAddress, Value};
use crate::blk::Block;
use crate::bus::{self, Mux};
use crate::dev::Device;
use crate::Address;

/// Bus mask.
///
/// # Usage
///
/// The `Mask` adapter...
#[derive(Debug)]
pub struct Mask<T, Idx, V>(Vec<Layer<T, Idx, V>>)
where
    T: Mux<Idx, V>,
    Idx: Value,
    V: Value;

impl<T, Idx, V> Mask<T, Idx, V>
where
    T: Mux<Idx, V>,
    Idx: Value,
    V: Value,
{
    /// Constructs a new `Mask`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the layer residing at `index`.
    #[must_use]
    pub fn layer(&self, index: usize) -> Option<&Layer<T, Idx, V>> {
        self.0.get(index)
    }

    /// Returns a mutable reference to the layer residing at `index`.
    pub fn layer_mut(&mut self, index: usize) -> Option<&mut Layer<T, Idx, V>> {
        self.0.get_mut(index)
    }

    /// Inserts a layer at position `index` within the mask.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    pub fn insert(&mut self, index: usize, bus: T) {
        self.0.insert(index, Layer::new(bus));
    }

    /// Removes and returns the layer at position `index` within the mask.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> T {
        self.0.remove(index).bus
    }

    /// Appends a layer to the back of the mask.
    pub fn push(&mut self, bus: T) {
        self.0.push(Layer::new(bus));
    }

    /// Removes the last layer from the mask and returns it.
    pub fn pop(&mut self) -> Option<T> {
        self.0.pop().map(|layer| layer.bus)
    }

    /// Reverses the order of layers in the mask, in place.
    pub fn reverse(&mut self) {
        self.0.reverse();
    }
}

impl<T, Idx, V> Address<Idx, V> for Mask<T, Idx, V>
where
    T: Mux<Idx, V>,
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

impl<T, Idx, V> TryAddress<Idx, V> for Mask<T, Idx, V>
where
    T: Mux<Idx, V>,
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

impl<T, Idx, V> Block for Mask<T, Idx, V>
where
    T: Mux<Idx, V>,
    Idx: Value,
    V: Value,
{
}

impl<T, Idx, V> Default for Mask<T, Idx, V>
where
    T: Mux<Idx, V>,
    Idx: Value,
    V: Value,
{
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl<T, Idx, V> Device<Idx, V> for Mask<T, Idx, V>
where
    T: Mux<Idx, V>,
    Idx: Value,
    V: Value,
{
}

#[derive(Debug)]
pub struct Layer<T, Idx, V>
where
    T: Mux<Idx, V>,
    Idx: Value,
    V: Value,
{
    pub bus: T,
    pub skip: bool,
    phantom: PhantomData<(Idx, V)>,
}

impl<T, Idx, V> Layer<T, Idx, V>
where
    T: Mux<Idx, V>,
    Idx: Value,
    V: Value,
{
    #[must_use]
    pub fn new(bus: T) -> Self {
        Self {
            bus,
            skip: false,
            phantom: PhantomData,
        }
    }
}

/// A type specifying general categories of [`Mask`] error.
pub type Error<Idx> = bus::Error<Idx>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::Dynamic;
    use crate::mem::Ram;

    type Bus = crate::bus::Bus<u16, u8>;

    fn setup() -> Mask<Bus, u16, u8> {
        // Create a new mask
        let mut mask = Mask::new();
        // Populate mask with layers
        for (range, value) in [
            // [aaaaaaaa        ]
            (0x00..=0x7f, 0xaa),
            // [  bbbbbbbb      ]
            (0x20..=0x9f, 0xbb),
            // [    cccccccc    ]
            (0x40..=0xbf, 0xcc),
            // [      dddddddd  ]
            (0x60..=0xdf, 0xdd),
            // [        eeeeeeee]
            (0x80..=0xff, 0xee),
        ] {
            // Define bus
            let mut bus = Bus::new();
            // Declare device
            let dev: Dynamic<u16, u8> = Ram::from(&[value; 0x80]).to_dynamic();
            bus.map(range, dev);
            // Add layer
            mask.push(bus);
        }
        // Reverse the mask
        mask.reverse();
        // [aabbccddeeeeeeee]
        mask
    }

    #[test]
    fn new_works() {
        let _: Mask<Bus, u16, u8> = Mask::new();
    }

    #[test]
    fn address_read_works() {
        let mask = setup();
        (0x00..=0x1f).for_each(|index| {
            assert_eq!(mask.read(index), 0xaa);
        });
        (0x20..=0x3f).for_each(|index| {
            assert_eq!(mask.read(index), 0xbb);
        });
        (0x40..=0x5f).for_each(|index| {
            assert_eq!(mask.read(index), 0xcc);
        });
        (0x60..=0x7f).for_each(|index| {
            assert_eq!(mask.read(index), 0xdd);
        });
        (0x80..=0xff).for_each(|index| {
            assert_eq!(mask.read(index), 0xee);
        });
    }
}
