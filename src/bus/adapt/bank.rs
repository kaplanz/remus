use crate::arch::{Address, Value};
use crate::blk::Block;
use crate::dev::{Device, Dynamic};

/// Device bank.
///
/// # Usage
///
/// The `Bank` device adapter provides a switchable bank of devices to be used
/// when performing [`Device`] operations.
///
/// As it is simply a wrapper, its fields are public can be accessed directly.
#[derive(Debug, Default)]
pub struct Bank<Idx, V>
where
    Idx: Value,
    V: Value,
{
    sel: usize,
    vec: Vec<Dynamic<Idx, V>>,
}

impl<Idx, V> Bank<Idx, V>
where
    Idx: Value,
    V: Value,
{
    /// Constructs a new, empty `Bank`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the selected device.
    #[must_use]
    pub fn get(&self) -> usize {
        self.sel
    }

    /// Sets the selected device.
    pub fn set(&mut self, sel: usize) {
        self.sel = sel;
    }

    /// Appends a device to the back of a bank.
    pub fn add(&mut self, dev: Dynamic<Idx, V>) {
        self.vec.push(dev);
    }

    /// Clears the bank, removing all devices.
    pub fn clear(&mut self) {
        self.vec.clear();
    }

    /// Inserts an device at position `index` within the bank, shifting all
    /// devices after it to the right.
    pub fn insert(&mut self, index: usize, dev: Dynamic<Idx, V>) {
        self.vec.insert(index, dev);
    }

    /// Removes and returns the device at position `index` within the bank,
    /// shifting all devices after it to the left.
    pub fn remove(&mut self, index: usize) -> Dynamic<Idx, V> {
        self.vec.remove(index)
    }

    /// Returns the number of banks.
    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

impl<Idx, V> Address<Idx, V> for Bank<Idx, V>
where
    Idx: Value,
    V: Value,
{
    fn read(&self, index: Idx) -> V {
        self.vec[self.sel].read(index)
    }

    fn write(&mut self, index: Idx, value: V) {
        self.vec[self.sel].write(index, value);
    }
}

impl<Idx, V> Block for Bank<Idx, V>
where
    Idx: Value,
    V: Value,
{
    fn reset(&mut self) {
        self.sel = 0;
        for bank in &mut self.vec {
            bank.reset();
        }
    }
}

impl<Idx, V> Device<Idx, V> for Bank<Idx, V>
where
    Idx: Value,
    V: Value,
{
}

impl<Idx, V> From<&[Dynamic<Idx, V>]> for Bank<Idx, V>
where
    Idx: Value,
    V: Value,
{
    fn from(banks: &[Dynamic<Idx, V>]) -> Self {
        Self {
            vec: Vec::from(banks),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::{Null, Random};
    use crate::mem::Ram;

    fn setup() -> Bank<usize, u8> {
        let mut bank = Bank::new();
        let ram = Ram::from(&[0x55; 0x100]).to_dynamic();
        let null = Null::<u8>::new().to_dynamic();
        let random = Random::<u8, 0x100>::new().to_dynamic();
        bank.vec.extend([ram, null, random]);
        bank
    }

    #[test]
    fn new_works() {
        let _ = Bank::<usize, u8>::new();
    }

    #[test]
    fn address_read_works() {
        let mut bank = setup();
        // Test bank 0
        bank.sel = 0;
        (0x00..=0xff).for_each(|index| assert_eq!(bank.read(index), 0x55));
        // Test bank 2
        bank.sel = 2;
        (0x00..=0xff).for_each(|index| {
            let _ = bank.read(index);
        });
    }

    #[test]
    fn address_write_works() {
        let mut bank = setup();
        // Test bank 0
        bank.sel = 0;
        (0x00..=0xff).for_each(|index| bank.write(index, 0xaa));
        (0x00..=0xff).for_each(|index| assert_eq!(bank.read(index), 0xaa));
        // Test bank 2
        bank.sel = 2;
        (0x00..=0xff).for_each(|index| bank.write(index, 0xaa));
        // NOTE: For all intents and purposes, this should never fail. If it
        //       does, one of two things happened:
        //       1. You broke something either in Bank or Random
        //       2. You broke this test
        //       3. You broke probability, all hope is lost
        assert!((0x00..=0xff)
            .map(|index| bank.read(index))
            .any(|value| value != 0xaa));
    }
}
