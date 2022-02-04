//! Memory interface bus.
//!
//! # Usage
//!
//! The [`Bus`] trait allows the user to mount another [`Device`] to
//! anywhere within the address space. As it itself implements [`Device`], it
//! may be mapped in a nested fashion.
//!
//! Together with the [`adapters`], [`Bus`] is the primary method of emulating
//! [memory-mapped I/O].
//!
//! [memory-mapped I/O]: https://en.wikipedia.org/wiki/Memory-mapped_I/O

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::rc::Rc;

use crate::blk::Block;
use crate::dev::Device;

pub mod adapters;

type DynDevice = Rc<RefCell<dyn Device>>;

/// Memory interface bus.
#[derive(Debug, Default)]
pub struct Bus {
    maps: BTreeMap<usize, Vec<DynDevice>>,
}

impl Bus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn map(&mut self, base: usize, dev: DynDevice) {
        self.maps.entry(base).or_default().push(dev);
    }

    pub fn unmap(&mut self, base: usize, dev: DynDevice) -> Option<DynDevice> {
        let devs = self.maps.get_mut(&base)?;
        let index = devs.iter().position(|this| Rc::ptr_eq(this, &dev))?;
        Some(devs.remove(index))
    }

    fn at(&self, index: usize) -> Option<(&usize, &DynDevice)> {
        self.maps
            .range(..=index)
            .rev()
            .flat_map(|(base, devs)| std::iter::repeat(base).zip(devs))
            .find(|(&base, dev)| dev.borrow().contains(index - base))
    }

    fn at_mut(&mut self, index: usize) -> Option<(&usize, &mut DynDevice)> {
        self.maps
            .range_mut(..=index)
            .rev()
            .flat_map(|(base, devs)| std::iter::repeat(base).zip(devs))
            .find(|(&base, dev)| dev.borrow().contains(index - base))
    }
}

impl Block for Bus {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Device for Bus {
    fn contains(&self, index: usize) -> bool {
        match self.at(index) {
            Some((base, dev)) => dev.borrow().contains(index - base),
            None => false,
        }
    }

    fn read(&self, index: usize) -> u8 {
        let (base, dev) = self.at(index).unwrap();
        dev.borrow().read(index - base)
    }

    fn write(&mut self, index: usize, byte: u8) {
        let (base, dev) = self.at_mut(index).unwrap();
        dev.borrow_mut().write(index - base, byte);
    }
}

impl<const N: usize> From<[(usize, Vec<DynDevice>); N]> for Bus {
    fn from(arr: [(usize, Vec<DynDevice>); N]) -> Self {
        Self { maps: arr.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Ram;

    fn setup() -> Bus {
        Bus::from([
            (0x000, vec![Rc::new(RefCell::new(vec![1; 0x100]))]),
            (0x100, vec![Rc::new(RefCell::new(vec![2; 0x100]))]),
            (0x200, vec![Rc::new(RefCell::new(vec![3; 0x100]))]),
        ] as [(usize, Vec<DynDevice>); 3])
    }

    #[test]
    fn new_works() {
        let bus = Bus::new();
        assert!(bus.maps.is_empty());
    }

    #[test]
    fn from_works() {
        let _ = Bus::from([
            (0x000, vec![Rc::new(RefCell::new(Box::from([0; 0x100])))]),
            (0x100, vec![Rc::new(RefCell::new(vec![0; 0x100]))]),
            (0x200, vec![Rc::new(RefCell::new(Ram::<0x100>::new()))]),
        ] as [(usize, Vec<DynDevice>); 3]);
    }

    #[test]
    fn map_works() {
        let mut bus = Bus::new();
        bus.map(0x000, Rc::new(RefCell::new(Box::from([0; 0x100]))));
        bus.map(0x100, Rc::new(RefCell::new(vec![0; 0x100])));
        bus.map(0x200, Rc::new(RefCell::new(Ram::<0x100>::new())));
    }

    #[test]
    #[should_panic]
    fn unmap_works() {
        let mut bus = Bus::new();
        let d0 = Rc::new(RefCell::new(Box::from([0; 0x100])));
        bus.map(0x000, d0.clone());
        bus.map(0x100, Rc::new(RefCell::new(vec![0; 0x100])));
        bus.map(0x200, Rc::new(RefCell::new(Ram::<0x100>::new())));
        bus.unmap(0x000, d0);
        bus.read(0x000);
    }

    #[test]
    fn device_contains_works() {
        // Let's create a mapping where a mapped sub-device has holes that
        // should be covered by another device, mapped elsewhere:
        // d0: [aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa]
        // d1: [                bbbbbbbb    cccc]
        let mut bus = Bus::new();
        // Add device 0
        let d0 = Ram::<0x1000>::from(&[0xaa; 0x1000]);
        bus.map(0x0000, Rc::new(RefCell::new(d0)));
        // Add device 1
        let mut d1 = Bus::new();
        d1.map(
            0x0000,
            Rc::new(RefCell::new(Ram::<0x0400>::from(&[0xbb; 0x0400]))),
        );
        d1.map(
            0x0600,
            Rc::new(RefCell::new(Ram::<0x0200>::from(&[0xcc; 0x0200]))),
        );
        bus.map(0x0800, Rc::new(RefCell::new(d1)));

        // Let's create a relatively complicated overlapping bus:
        // d0: [                a                              ]
        // d1: [                 bb                            ]
        // d2: [                   cccc                        ]
        // d3: [                       ddddddddd               ]
        // d4: [eeeeeeeeeeeeeeee                               ]
        // d5: [ffffffffffffffffffffffffffffffffffffffffffff...]
        let mut bus = Bus::new();

        // Add device 0
        const N0: usize = 0x0;
        const A0: usize = 0x1000;
        let d0 = Ram::<N0>::from(&[0xaa; N0]);
        bus.map(A0, Rc::new(RefCell::new(d0)));
        (A4..A0).for_each(|addr| assert!(!bus.contains(addr)));
        (A0..A0 + N0).for_each(|addr| assert!(bus.contains(addr)));
        (A0 + N0..N5).for_each(|addr| assert!(!bus.contains(addr)));

        // Add device 1
        const N1: usize = 0x1;
        const A1: usize = A0 + N0;
        let d1 = Ram::<N1>::from(&[0xbb; N1]);
        bus.map(A1, Rc::new(RefCell::new(d1)));
        (A4..A0).for_each(|addr| assert!(!bus.contains(addr)));
        (A0..A1 + N1).for_each(|addr| assert!(bus.contains(addr)));
        (A1 + N1..N5).for_each(|addr| assert!(!bus.contains(addr)));

        // Add device 2
        const N2: usize = 0x10;
        const A2: usize = A1 + N1;
        let d2 = Ram::<N2>::from(&[0xcc; N2]);
        bus.map(A2, Rc::new(RefCell::new(d2)));
        (A4..A0).for_each(|addr| assert!(!bus.contains(addr)));
        (A0..A2 + N2).for_each(|addr| assert!(bus.contains(addr)));
        (A2 + N2..N5).for_each(|addr| assert!(!bus.contains(addr)));

        // Add device 3
        const N3: usize = 0x100;
        const A3: usize = A2 + N2;
        let d3 = Ram::<N3>::from(&[0xdd; N3]);
        bus.map(A3, Rc::new(RefCell::new(d3)));
        (A4..A0).for_each(|addr| assert!(!bus.contains(addr)));
        (A0..A3 + N3).for_each(|addr| assert!(bus.contains(addr)));
        (A3 + N3..N5).for_each(|addr| assert!(!bus.contains(addr)));

        // Add device 4
        const N4: usize = 0x1000;
        const A4: usize = 0x0;
        let d4 = Ram::<N4>::from(&[0xee; N4]);
        bus.map(A4, Rc::new(RefCell::new(d4)));
        (A4..A3 + N3).for_each(|addr| assert!(bus.contains(addr)));
        (A3 + N3..N5).for_each(|addr| assert!(!bus.contains(addr)));

        // Add device 5
        const N5: usize = 0x2000;
        const A5: usize = A4;
        let d5 = Ram::<N5>::from(&[0xff; N5]);
        bus.map(A5, Rc::new(RefCell::new(d5)));
        (A5..N5).for_each(|addr| assert!(bus.contains(addr)));
    }

    #[test]
    fn device_read_mapped_works() {
        let bus = setup();
        (0x000..0x100).for_each(|i| assert_eq!(bus.read(i), 1));
        (0x100..0x200).for_each(|i| assert_eq!(bus.read(i), 2));
        (0x200..0x300).for_each(|i| assert_eq!(bus.read(i), 3));
    }

    #[test]
    #[should_panic]
    fn device_read_unmapped_panics() {
        let bus = setup();
        bus.read(0x301);
    }

    #[test]
    fn device_write_mapped_works() {
        let mut bus = setup();
        (0x000..0x300).for_each(|i| bus.write(i, 4));
        (0x000..0x300).for_each(|i| assert_eq!(bus.read(i), 4));
    }

    #[test]
    #[should_panic]
    fn device_write_unmapped_panics() {
        let mut bus = setup();
        bus.write(0x301, 4);
    }

    #[test]
    fn device_read_write_holes_mapped_works() {
        // Let's create a mapping where a mapped sub-device has holes that
        // should be covered by another device, mapped elsewhere:
        // d0: [aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa]
        // d1: [                bbbbbbbb    cccc]
        let mut bus = Bus::new();
        // Add device 0
        let d0 = Ram::<0x1000>::from(&[0xaa; 0x1000]);
        bus.map(0x0000, Rc::new(RefCell::new(d0)));
        // Add device 1
        let mut d1 = Bus::new();
        d1.map(
            0x0000,
            Rc::new(RefCell::new(Ram::<0x0400>::from(&[0xbb; 0x0400]))),
        );
        d1.map(
            0x0600,
            Rc::new(RefCell::new(Ram::<0x0200>::from(&[0xcc; 0x0200]))),
        );
        bus.map(0x0800, Rc::new(RefCell::new(d1)));

        // Check if it is accessed properly...
        (0x0000..=0x07ff)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xaa);
        (0x0800..=0x0bff)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xbb);
        (0x0c00..=0x0dff)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xaa);
        (0x0e00..=0x0fff)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xcc);
    }

    #[test]
    fn device_read_write_overlapping_mapped_works() {
        // Let's create a relatively complicated overlapping bus:
        // d0: [                a                              ]
        // d1: [                 bb                            ]
        // d2: [                   cccc                        ]
        // d3: [                       ddddddddd               ]
        // d4: [eeeeeeeeeeeeeeee                               ]
        // d5: [ffffffffffffffffffffffffffffffffffffffffffff...]
        let mut bus = Bus::new();
        // Add device 0
        const N0: usize = 0x0;
        const A0: usize = 0x1000;
        let d0 = Ram::<N0>::from(&[0xaa; N0]);
        bus.map(A0, Rc::new(RefCell::new(d0)));
        // Add device 1
        const N1: usize = 0x1;
        const A1: usize = A0 + N0;
        let d1 = Ram::<N1>::from(&[0xbb; N1]);
        bus.map(A1, Rc::new(RefCell::new(d1)));
        // Add device 2
        const N2: usize = 0x10;
        const A2: usize = A1 + N1;
        let d2 = Ram::<N2>::from(&[0xcc; N2]);
        bus.map(A2, Rc::new(RefCell::new(d2)));
        // Add device 3
        const N3: usize = 0x100;
        const A3: usize = A2 + N2;
        let d3 = Ram::<N3>::from(&[0xdd; N3]);
        bus.map(A3, Rc::new(RefCell::new(d3)));
        // Add device 4
        const N4: usize = 0x1000;
        const A4: usize = 0x0000;
        let d4 = Ram::<N4>::from(&[0xee; N4]);
        bus.map(A4, Rc::new(RefCell::new(d4)));
        // Add device 5
        const N5: usize = 0x10000;
        const A5: usize = A4;
        let d5 = Ram::<N5>::from(&[0xff; N5]);
        bus.map(A5, Rc::new(RefCell::new(d5)));

        // Check if it is accessed properly...
        (0x0..A0)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xaa);
        (0x0..A1)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xbb);
        (0x0..A2)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xcc);
        (0x0..A3)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xdd);
        (A4..A4 + N4)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xee);
        (A5..A5 + N5)
            .map(|addr| bus.read(addr))
            .all(|byte| byte == 0xff);
    }
}
