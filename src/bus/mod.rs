//! Memory interface.
//!
//! # Usage
//!
//! The [`Bus`] struct allows the user to mount another [`Device`] to
//! anywhere within the address space. As it itself implements `Device`, it
//! may be mapped in a nested fashion.
//!
//! Together with the [adapters](self::adapt), `Bus` is the primary method of
//! emulating [memory-mapped I/O].
//!
//! [memory-mapped I/O]: https://en.wikipedia.org/wiki/Memory-mapped_I/O

use std::fmt::Debug;
use std::ops::{Index, RangeInclusive};

use self::map::Bus as BusMap;
use crate::arch::{Address, Value};
use crate::blk::Block;
use crate::dev::{Device, Dynamic};

mod map;

pub mod adapt;

type Range<Idx> = RangeInclusive<Idx>;

/// Address [bus][bus].
///
/// [bus]: https://en.wikipedia.org/wiki/Bus_(computing)
#[derive(Debug, Default)]
pub struct Bus<Idx, V>
where
    Idx: Value,
    V: Value,
{
    maps: BusMap<Idx, Dynamic<Idx, V>>,
}

impl<Idx, V> Bus<Idx, V>
where
    Idx: Value,
    V: Value,
{
    /// Constructs a new, empty `Bus`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears the bus, removing all devices.
    pub fn clear(&mut self) {
        self.maps.clear();
    }

    /// Maps a device at the provided `base` address in the bus.
    pub fn map(&mut self, range: Range<Idx>, dev: Dynamic<Idx, V>) {
        self.maps.map(range, dev);
    }

    /// Unmaps and returns the matching device in the bus.
    ///
    /// Returns `None` if no matching device was found (and unmapped).
    pub fn unmap(&mut self, dev: &Dynamic<Idx, V>) -> Option<Dynamic<Idx, V>> {
        // self.maps.unmap(dev)
        todo!("{dev:?}")
    }

    pub fn get(&self, index: Idx) -> Option<&Dynamic<Idx, V>> {
        self.maps.get(index).map(|map| &map.entry)
    }
}

impl<Idx, V> Address<Idx, V> for Bus<Idx, V>
where
    Idx: Value,
    V: Value,
{
    fn read(&self, index: Idx) -> V {
        let map = self.maps.get(index).unwrap();
        map.entry.read(index - map.base())
    }

    fn write(&mut self, index: Idx, value: V) {
        let map = self.maps.get(index).unwrap();
        map.entry.borrow_mut().write(index - map.base(), value);
    }
}

impl<Idx, V> Block for Bus<Idx, V>
where
    Idx: Value,
    V: Value,
{
}

impl<Idx, V> Device<Idx, V> for Bus<Idx, V>
where
    Idx: Value,
    V: Value,
{
}

impl<Idx, V> Index<Idx> for Bus<Idx, V>
where
    Idx: Value,
    V: Value,
{
    type Output = Dynamic<Idx, V>;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.maps.get(index).unwrap().entry
    }
}

impl<Idx, V, const N: usize> From<[(Range<Idx>, Dynamic<Idx, V>); N]> for Bus<Idx, V>
where
    Idx: Value,
    V: Value,
{
    fn from(arr: [(Range<Idx>, Dynamic<Idx, V>); N]) -> Self {
        let mut this = Self::default();
        for (range, dev) in arr {
            this.map(range, dev);
        }
        this
    }
}

#[allow(clippy::items_after_statements)]
#[allow(clippy::range_plus_one)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Ram;

    fn setup() -> Bus<usize, u8> {
        Bus::from([
            (0x000..=0x0ff, Ram::from(&[0; 0x100]).to_dynamic()),
            (0x100..=0x1ff, Ram::from(&[1; 0x100]).to_dynamic()),
            (0x200..=0x2ff, Ram::from(&[2; 0x100]).to_dynamic()),
        ])
    }

    #[test]
    fn new_works() {
        let bus = Bus::<usize, u8>::new();
        assert_eq!(bus.maps.iter().count(), 0);
    }

    #[test]
    fn from_works() {
        setup();
    }

    #[test]
    fn clear_works() {
        let mut bus = setup();
        bus.clear();
        assert_eq!(bus.maps.iter().count(), 0);
    }

    #[test]
    fn map_works() {
        let bus = setup();
        assert!((0x000..=0x0ff)
            .map(|idx| bus.read(idx))
            .all(|value| value == 0));
        assert!((0x100..=0x1ff)
            .map(|idx| bus.read(idx))
            .all(|value| value == 1));
        assert!((0x200..=0x2ff)
            .map(|idx| bus.read(idx))
            .all(|value| value == 2));
    }

    #[test]
    #[should_panic]
    fn unmap_works() {
        let mut bus = Bus::new();
        let dev: Dynamic<usize, u8> = Ram::from(&[0; 0x100]).to_dynamic();
        bus.map(0x000..=0x0ff, dev.clone());
        assert_eq!(bus.unmap(&dev), Some(dev));
        bus.read(0x000);
    }

    #[test]
    fn address_read_mapped_works() {
        let bus = setup();
        (0x000..0x100).for_each(|i| assert_eq!(bus.read(i), 0));
        (0x100..0x200).for_each(|i| assert_eq!(bus.read(i), 1));
        (0x200..0x300).for_each(|i| assert_eq!(bus.read(i), 2));
    }

    #[test]
    #[should_panic]
    fn address_read_unmapped_panics() {
        let bus = setup();
        bus.read(0x301);
    }

    #[test]
    fn address_write_mapped_works() {
        let mut bus = setup();
        (0x000..0x300).for_each(|i| bus.write(i, 4));
        (0x000..0x300).for_each(|i| assert_eq!(bus.read(i), 4));
    }

    #[test]
    #[should_panic]
    fn address_write_unmapped_panics() {
        let mut bus = setup();
        bus.write(0x301, 4);
    }

    #[allow(clippy::range_minus_one)]
    #[allow(clippy::reversed_empty_ranges)]
    #[test]
    fn address_read_write_overlapping_mapped_works() {
        // Let's create a relatively complicated overlapping bus:
        //     ┌─────────────────────────────────────────────────┐
        // D0: │                 a                               │
        // D1: │                  bb                             │
        // D2: │                    cccc                         │
        // D3: │                        ddddddddd                │
        // D4: │ eeeeeeeeeeeeeeee                                │
        // D5: │ ffffffffffffffffffffffffffffffffffffffffffff... │
        //     ├─────────────────────────────────────────────────┤
        //     │ eeeeeeeeeeeeeeeeabbccccdddddddddffffffffffff... │
        //     └─────────────────────────────────────────────────┘
        let mut bus = Bus::new();
        // Device 0
        const N0: usize = 1;
        const A0: usize = 0x1000;
        let d0 = Ram::from(&[0xaa; N0]);
        bus.map(A0..=A0 + N0 - 1, d0.to_dynamic());
        // Device 1
        const N1: usize = 2;
        const A1: usize = A0 + N0;
        let d1 = Ram::from(&[0xbb; N1]);
        bus.map(A1..=A1 + N1 - 1, d1.to_dynamic());
        // Device 2
        const N2: usize = 4;
        const A2: usize = A1 + N1;
        let d2 = Ram::from(&[0xcc; N2]);
        bus.map(A2..=A2 + N2 - 1, d2.to_dynamic());
        // Device 3
        const N3: usize = 8;
        const A3: usize = A2 + N2;
        let d3 = Ram::from(&[0xdd; N3]);
        bus.map(A3..=A3 + N3 - 1, d3.to_dynamic());
        // Device 4
        const N4: usize = 16;
        const A4: usize = 0;
        let d4 = Ram::from(&[0xee; N4]);
        bus.map(A4..=A4 + N4 - 1, (d4).to_dynamic());
        // Device 5
        const N5: usize = 128;
        const A5: usize = A4;
        let d5 = Ram::from(&[0xff; N5]);
        bus.map(A5..=A5 + N5 - 1, (d5).to_dynamic());

        // Check if it is accessed properly...
        assert!((A0..A0 + N0)
            .map(|index| bus.read(index))
            .all(|byte| byte == 0xaa));
        assert!((A1..A1 + N1)
            .map(|index| bus.read(index))
            .all(|byte| byte == 0xbb));
        assert!((A2..A2 + N2)
            .map(|index| bus.read(index))
            .all(|byte| byte == 0xcc));
        assert!((A3..A3 + N3)
            .map(|index| bus.read(index))
            .all(|byte| byte == 0xdd));
        assert!((A4..A4 + N4)
            .map(|index| bus.read(index))
            .all(|byte| byte == 0xee));
        assert!((A3 + N3..A5 + N5)
            .map(|index| bus.read(index))
            .all(|byte| byte == 0xff));
    }
}
