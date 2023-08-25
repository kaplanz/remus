//! Memory interface.
//!
//! # Usage
//!
//! The [`Bus`] trait allows the user to mount another [`Device`] to
//! anywhere within the address space. As it itself implements `Device`, it
//! may be mapped in a nested fashion.
//!
//! Together with the [adapters](self::adapt), `Bus` is the primary method of emulating
//! [memory-mapped I/O].
//!
//! [memory-mapped I/O]: https://en.wikipedia.org/wiki/Memory-mapped_I/O

use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::arc::Address;
use crate::blk::Block;
use crate::dev::{Device, Dynamic};

pub mod adapt;

/// Address [bus][bus].
///
/// [bus]: https://en.wikipedia.org/wiki/Bus_(computing)
#[derive(Debug, Default)]
pub struct Bus {
    maps: BTreeMap<usize, Vec<Dynamic>>,
}

impl Bus {
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
    pub fn map(&mut self, base: usize, dev: Dynamic) {
        let devs = self.maps.entry(base).or_default();
        devs.push(dev);
        devs.sort_by_key(Device::len);
    }

    /// Unmaps and returns the matching device at position `index` within the
    /// bus.
    ///
    /// Returns `None` if no matching device was found (and unmapped).
    pub fn unmap(&mut self, base: usize, dev: &Dynamic) -> Option<Dynamic> {
        let devs = self.maps.get_mut(&base)?;
        let index = devs.iter().position(|this| this == dev)?;
        Some(devs.remove(index))
    }

    /// Gets the `base` and `SharedDevice` mapped at `index`.
    fn at(&self, index: usize) -> Option<(&usize, &Dynamic)> {
        self.maps
            .range(..=index)
            .rev()
            .flat_map(|(base, devs)| std::iter::repeat(base).zip(devs))
            .find(|(&base, dev)| dev.contains(index - base))
    }

    /// Mutably gets the `base` and `SharedDevice` mapped at `index`.
    fn at_mut(&mut self, index: usize) -> Option<(&usize, &mut Dynamic)> {
        self.maps
            .range_mut(..=index)
            .rev()
            .flat_map(|(base, devs)| std::iter::repeat(base).zip(devs))
            .find(|(&base, dev)| dev.contains(index - base))
    }
}

impl Address for Bus {
    fn read(&self, index: usize) -> u8 {
        let (base, dev) = self.at(index).unwrap();
        dev.read(index - base)
    }

    fn write(&mut self, index: usize, byte: u8) {
        let (base, dev) = self.at_mut(index).unwrap();
        dev.write(index - base, byte);
    }
}

impl Block for Bus {
    fn reset(&mut self) {
        for dev in self.maps.iter_mut().flat_map(|(_, devs)| devs) {
            dev.reset();
        }
    }
}

impl Device for Bus {
    fn contains(&self, index: usize) -> bool {
        match self.at(index) {
            Some((base, dev)) => dev.contains(index - base),
            None => false,
        }
    }

    fn len(&self) -> usize {
        let start = *self.maps.keys().next().unwrap_or(&0);
        let end = self
            .maps
            .iter()
            .flat_map(|(base, devs)| std::iter::repeat(base).zip(devs))
            .map(|(&base, dev)| base + dev.len())
            .max()
            .unwrap_or(0);
        end.saturating_sub(start)
    }
}

impl<const N: usize> From<[(usize, Dynamic); N]> for Bus {
    fn from(arr: [(usize, Dynamic); N]) -> Self {
        let mut this = Self::default();
        for (addr, dev) in arr {
            this.map(addr, dev);
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

    fn setup() -> Bus {
        Bus::from([
            (0x000, Ram::<0x100>::new().to_dynamic()),
            (0x100, Ram::from(&[1; 0x100]).to_dynamic()),
            (0x200, Ram::from(&[2; 0x100]).to_dynamic()),
        ])
    }

    #[test]
    fn new_works() {
        let bus = Bus::new();
        assert!(bus.maps.is_empty());
    }

    #[test]
    fn from_works() {
        let _ = Bus::from([
            (0x000, Ram::<0x100>::new().to_dynamic()),
            (0x100, Ram::from(&[1; 0x100]).to_dynamic()),
            (0x200, Ram::from(&[2; 0x100]).to_dynamic()),
        ]);
    }

    #[test]
    fn clear_works() {
        let mut bus = setup();
        bus.clear();
        assert_eq!(bus.maps.len(), 0);
    }

    #[test]
    fn map_works() {
        let mut bus = Bus::new();
        bus.map(0x000, Ram::from(&[0; 0x100]).to_dynamic());
        bus.map(0x100, Ram::from(&[1; 0x100]).to_dynamic());
        bus.map(0x200, Ram::<0x100>::new().to_dynamic());
    }

    #[test]
    #[should_panic]
    fn unmap_works() {
        let mut bus = Bus::new();
        let d0 = Ram::from(&[0; 0x100]).to_dynamic();
        bus.map(0x000, d0.clone());
        bus.map(0x100, Ram::from(&[0xaa; 0x100]).to_dynamic());
        bus.map(0x200, Ram::<0x100>::new().to_dynamic());
        bus.unmap(0x000, &d0);
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

    #[test]
    fn address_read_write_holes_mapped_works() {
        // Let's create a mapping where a mapped sub-device has holes that
        // should be covered by another device, mapped elsewhere:
        //     ┌──────────────────────────────────┐
        // D0: │ 00000000000000000000000000000000 │
        // D1: │                 11111111    2222 │
        //     ├──────────────────────────────────┤
        //     │ 00000000000000001111111100002222 │
        //     └──────────────────────────────────┘
        let mut bus = Bus::new();
        // Add device 0
        let d0 = Ram::<0x1000>::from(&[0xaa; 0x1000]);
        bus.map(0x0000, d0.to_dynamic());
        // Add device 1
        let mut d1 = Bus::new();
        d1.map(0x0000, Ram::<0x0400>::from(&[0xbb; 0x0400]).to_dynamic());
        d1.map(0x0600, Ram::<0x0200>::from(&[0xcc; 0x0200]).to_dynamic());
        bus.map(0x0800, d1.to_dynamic());

        // Check if it is accessed properly...
        assert!((0x0000..=0x07ff)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xaa));
        assert!((0x0800..=0x0bff)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xbb));
        assert!((0x0c00..=0x0dff)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xaa));
        assert!((0x0e00..=0x0fff)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xcc));
    }

    #[test]
    fn address_read_write_overlapping_mapped_works() {
        // Let's create a relatively complicated overlapping bus:
        //     ┌─────────────────────────────────────────────────┐
        // D0: │                 0                               │
        // D1: │                  11                             │
        // D2: │                    2222                         │
        // D3: │                        333333333                │
        // D4: │ 4444444444444444                                │
        // D5: │ 55555555555555555555555555555555555555555555... │
        //     ├─────────────────────────────────────────────────┤
        //     │ 44444444444444440112222333333333555555555555... │
        //     └─────────────────────────────────────────────────┘
        let mut bus = Bus::new();
        // Add device 0
        const N0: usize = 0x0;
        const A0: usize = 0x1000;
        let d0 = Ram::<N0>::from(&[0xaa; N0]);
        bus.map(A0, d0.to_dynamic());
        // Add device 1
        const N1: usize = 0x1;
        const A1: usize = A0 + N0;
        let d1 = Ram::<N1>::from(&[0xbb; N1]);
        bus.map(A1, d1.to_dynamic());
        // Add device 2
        const N2: usize = 0x10;
        const A2: usize = A1 + N1;
        let d2 = Ram::<N2>::from(&[0xcc; N2]);
        bus.map(A2, d2.to_dynamic());
        // Add device 3
        const N3: usize = 0x100;
        const A3: usize = A2 + N2;
        let d3 = Ram::<N3>::from(&[0xdd; N3]);
        bus.map(A3, d3.to_dynamic());
        // Add device 4
        const N4: usize = 0x1000;
        const A4: usize = 0x0000;
        let d4 = Ram::<N4>::from(&[0xee; N4]);
        bus.map(A4, (d4).to_dynamic());
        // Add device 5
        const N5: usize = 0x10000;
        const A5: usize = A4;
        let d5 = Ram::<N5>::from(&[0xff; N5]);
        bus.map(A5, (d5).to_dynamic());

        // Check if it is accessed properly...
        assert!((A0..A0 + N0)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xaa));
        assert!((A1..A1 + N1)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xbb));
        assert!((A2..A2 + N2)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xcc));
        assert!((A3..A3 + N3)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xdd));
        assert!((A4..A4 + N4)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xee));
        assert!((A3 + N3..A5 + N5)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xff));
    }

    #[test]
    fn block_reset_works() {
        let mut bus = setup();
        bus.reset();
        assert_eq!(bus.maps.len(), setup().maps.len());
    }

    #[test]
    fn device_contains_works() {
        // Let's create a mapping where a mapped sub-device has holes that
        // should be covered by another device, mapped elsewhere:
        //     ┌──────────────────────────────────┐
        // D0: │ 00000000000000000000000000000000 │
        // D1: │                 11111111    2222 │
        //     ├──────────────────────────────────┤
        //     │ 00000000000000001111111100002222 │
        //     └──────────────────────────────────┘
        let mut bus = Bus::new();
        // Add device 0
        let d0 = Ram::<0x1000>::from(&[0xaa; 0x1000]);
        bus.map(0x0000, d0.to_dynamic());
        // Add device 1
        let mut d1 = Bus::new();
        d1.map(0x0000, Ram::<0x0400>::from(&[0xbb; 0x0400]).to_dynamic());
        d1.map(0x0600, Ram::<0x0200>::from(&[0xcc; 0x0200]).to_dynamic());
        bus.map(0x0800, d1.to_dynamic());

        // Let's create a relatively complicated overlapping bus:
        //     ┌─────────────────────────────────────────────────┐
        // D0: │                 0                               │
        // D1: │                  11                             │
        // D2: │                    2222                         │
        // D3: │                        333333333                │
        // D4: │ 4444444444444444                                │
        // D5: │ 55555555555555555555555555555555555555555555... │
        //     ├─────────────────────────────────────────────────┤
        //     │ 44444444444444440112222333333333555555555555... │
        //     └─────────────────────────────────────────────────┘
        let mut bus = Bus::new();

        // Add device 0
        const N0: usize = 0x0;
        const A0: usize = 0x1000;
        let d0 = Ram::<N0>::from(&[0xaa; N0]);
        bus.map(A0, d0.to_dynamic());
        assert!((A4..A0).all(|addr| !bus.contains(addr)));
        assert!((A0..A0 + N0).all(|addr| bus.contains(addr)));
        assert!((A0 + N0..N5).all(|addr| !bus.contains(addr)));

        // Add device 1
        const N1: usize = 0x1;
        const A1: usize = A0 + N0;
        let d1 = Ram::<N1>::from(&[0xbb; N1]);
        bus.map(A1, d1.to_dynamic());
        assert!((A4..A0).all(|addr| !bus.contains(addr)));
        assert!((A0..A1 + N1).all(|addr| bus.contains(addr)));
        assert!((A1 + N1..N5).all(|addr| !bus.contains(addr)));

        // Add device 2
        const N2: usize = 0x10;
        const A2: usize = A1 + N1;
        let d2 = Ram::<N2>::from(&[0xcc; N2]);
        bus.map(A2, d2.to_dynamic());
        assert!((A4..A0).all(|addr| !bus.contains(addr)));
        assert!((A0..A2 + N2).all(|addr| bus.contains(addr)));
        assert!((A2 + N2..N5).all(|addr| !bus.contains(addr)));

        // Add device 3
        const N3: usize = 0x100;
        const A3: usize = A2 + N2;
        let d3 = Ram::<N3>::from(&[0xdd; N3]);
        bus.map(A3, d3.to_dynamic());
        assert!((A4..A0).all(|addr| !bus.contains(addr)));
        assert!((A0..A3 + N3).all(|addr| bus.contains(addr)));
        assert!((A3 + N3..N5).all(|addr| !bus.contains(addr)));

        // Add device 4
        const N4: usize = 0x1000;
        const A4: usize = 0x0;
        let d4 = Ram::<N4>::from(&[0xee; N4]);
        bus.map(A4, d4.to_dynamic());
        assert!((A4..A3 + N3).all(|addr| bus.contains(addr)));
        assert!((A3 + N3..N5).all(|addr| !bus.contains(addr)));

        // Add device 5
        const N5: usize = 0x2000;
        const A5: usize = A4;
        let d5 = Ram::<N5>::from(&[0xff; N5]);
        bus.map(A5, d5.to_dynamic());
        assert!((A5..N5).all(|addr| bus.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        // Let's create a mapping where a mapped sub-device has holes that
        // should be covered by another device, mapped elsewhere:
        //     ┌──────────────────────────────────┐
        // D0: │ 00000000000000000000000000000000 │
        // D1: │                 11111111    2222 │
        //     ├──────────────────────────────────┤
        //     │ 00000000000000001111111100002222 │
        //     └──────────────────────────────────┘
        let mut bus = Bus::new();
        // Add device 0
        let d0 = Ram::<0x1000>::from(&[0xaa; 0x1000]);
        bus.map(0x0000, d0.to_dynamic());
        // Add device 1
        let mut d1 = Bus::new();
        d1.map(0x0000, Ram::<0x0400>::from(&[0xbb; 0x0400]).to_dynamic());
        d1.map(0x0600, Ram::<0x0200>::from(&[0xcc; 0x0200]).to_dynamic());
        bus.map(0x0800, d1.to_dynamic());

        // Let's create a relatively complicated overlapping bus:
        //     ┌─────────────────────────────────────────────────┐
        // D0: │                 0                               │
        // D1: │                  11                             │
        // D2: │                    2222                         │
        // D3: │                        333333333                │
        // D4: │ 4444444444444444                                │
        // D5: │ 55555555555555555555555555555555555555555555... │
        //     ├─────────────────────────────────────────────────┤
        //     │ 44444444444444440112222333333333555555555555... │
        //     └─────────────────────────────────────────────────┘
        let mut bus = Bus::new();

        // Add device 0
        const N0: usize = 0x0;
        const A0: usize = 0x1000;
        let d0 = Ram::<N0>::from(&[0xaa; N0]);
        bus.map(A0, d0.to_dynamic());
        assert_eq!(bus.len(), N0);

        // Add device 1
        const N1: usize = 0x1;
        const A1: usize = A0 + N0;
        let d1 = Ram::<N1>::from(&[0xbb; N1]);
        bus.map(A1, d1.to_dynamic());
        assert_eq!(bus.len(), N0 + N1);

        // Add device 2
        const N2: usize = 0x10;
        const A2: usize = A1 + N1;
        let d2 = Ram::<N2>::from(&[0xcc; N2]);
        bus.map(A2, d2.to_dynamic());
        assert_eq!(bus.len(), N0 + N1 + N2);

        // Add device 3
        const N3: usize = 0x100;
        const A3: usize = A2 + N2;
        let d3 = Ram::<N3>::from(&[0xdd; N3]);
        bus.map(A3, d3.to_dynamic());
        assert_eq!(bus.len(), N0 + N1 + N2 + N3);

        // Add device 4
        const N4: usize = 0x1000;
        const A4: usize = 0x0;
        let d4 = Ram::<N4>::from(&[0xee; N4]);
        bus.map(A4, d4.to_dynamic());
        assert_eq!(bus.len(), N4 + N0 + N1 + N2 + N3);

        // Add device 5
        const N5: usize = 0x2000;
        const A5: usize = A4;
        let d5 = Ram::<N5>::from(&[0xff; N5]);
        bus.map(A5, d5.to_dynamic());
        assert_eq!(bus.len(), N5);
    }
}
