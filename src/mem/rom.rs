use std::ops::Deref;

use crate::arc::Address;
use crate::blk::Block;
use crate::dev::Device;

/// Read-only memory model.
///
/// # Panics
///
/// Panics on [`Address::write`].
#[derive(Debug)]
pub struct Rom<const N: usize>(Box<[u8; N]>);

impl<const N: usize> Rom<N> {
    /// Constructs a new, empty `Rom<N>`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const N: usize> Address<u8> for Rom<N> {
    fn read(&self, index: usize) -> u8 {
        self[index]
    }

    /// # Panics
    ///
    /// Panics when attempting to write to a [`Rom`].
    fn write(&mut self, _index: usize, _value: u8) {
        panic!("called `Address::write()` on a `Rom`");
    }
}

impl<const N: usize> Block for Rom<N> {}

impl<const N: usize> Default for Rom<N> {
    fn default() -> Self {
        Self(
            vec![Default::default(); N]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        )
    }
}

impl<const N: usize> Deref for Rom<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<const N: usize> Device for Rom<N> {
    fn contains(&self, index: usize) -> bool {
        (0..self.len()).contains(&index)
    }

    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
}

impl<const N: usize> From<&[u8; N]> for Rom<N> {
    fn from(arr: &[u8; N]) -> Self {
        Self(Vec::from(&arr[..]).into_boxed_slice().try_into().unwrap())
    }
}

#[allow(clippy::cast_possible_truncation)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::Address;
    use crate::dev::Device;

    #[test]
    fn new_works() {
        let rom = Rom::<0x100>::new();
        assert!(rom.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_works() {
        const N: usize = 0x100;

        let arr = [0; N];
        let rom = Rom::<N>::from(&arr);
        assert_eq!(*rom, arr);

        let vec: Vec<u8> = (0..N).map(|x| x as u8).collect();
        let buf = vec.try_into().unwrap();
        let rom = Rom::<N>::from(&buf);
        assert_eq!(*rom, buf);
    }

    #[test]
    fn address_read_works() {
        let rom = Rom::<0x1>::from(&[0xaa]);
        assert_eq!(rom.read(0x0), 0xaa);
    }

    #[test]
    #[should_panic]
    fn address_write_panics() {
        let mut rom = Rom::<0x1>::from(&[0xaa]);
        rom.write(0x0, 0xaa);
    }

    #[test]
    #[allow(clippy::items_after_statements)]
    fn device_contains_works() {
        const N0: usize = 0x0;
        let rom = Rom::<N0>::new();
        (0..N0).for_each(|addr| assert!(rom.contains(addr)));

        const N1: usize = 0x1;
        let rom = Rom::<N1>::new();
        (0..N1).for_each(|addr| assert!(rom.contains(addr)));

        const N2: usize = 0x10;
        let rom = Rom::<N2>::new();
        (0..N2).for_each(|addr| assert!(rom.contains(addr)));

        const N3: usize = 0x100;
        let rom = Rom::<N3>::new();
        (0..N3).for_each(|addr| assert!(rom.contains(addr)));

        const N4: usize = 0x1000;
        let rom = Rom::<N4>::new();
        (0..N4).for_each(|addr| assert!(rom.contains(addr)));

        const N5: usize = 0x10000;
        let rom = Rom::<N5>::new();
        (0..N5).for_each(|addr| assert!(rom.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        assert_eq!(Rom::<0x0>::new().len(), 0x0);
        assert_eq!(Rom::<0x1>::new().len(), 0x1);
        assert_eq!(Rom::<0x10>::new().len(), 0x10);
        assert_eq!(Rom::<0x100>::new().len(), 0x100);
        assert_eq!(Rom::<0x1000>::new().len(), 0x1000);
        assert_eq!(Rom::<0x10000>::new().len(), 0x10000);
    }
}
