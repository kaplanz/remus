use crate::arch::Address;
use crate::blk::Block;
use crate::dev::Device;

/// Random-access memory model.
#[derive(Debug)]
pub struct Ram<const N: usize>(Box<[u8; N]>);

impl<const N: usize> Ram<N> {
    /// Constructs a new, empty `Ram<N>`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const N: usize> Address<u8> for Ram<N> {
    fn read(&self, index: usize) -> u8 {
        self.0[index]
    }

    fn write(&mut self, index: usize, value: u8) {
        self.0[index] = value;
    }
}

impl<const N: usize> Block for Ram<N> {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl<const N: usize> Default for Ram<N> {
    fn default() -> Self {
        Self(
            vec![Default::default(); N]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        )
    }
}

impl<const N: usize> Device for Ram<N> {
    fn contains(&self, index: usize) -> bool {
        (0..self.len()).contains(&index)
    }

    fn len(&self) -> usize {
        <[u8]>::len(&*self.0)
    }
}

impl<const N: usize> From<&[u8; N]> for Ram<N> {
    fn from(arr: &[u8; N]) -> Self {
        Self(Vec::from(&arr[..]).into_boxed_slice().try_into().unwrap())
    }
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::items_after_statements)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch::Address;
    use crate::dev::Device;

    #[test]
    fn new_works() {
        let ram = Ram::<0x100>::new();
        assert!(ram.0.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_works() {
        const N: usize = 0x100;

        let arr = [0; N];
        let ram = Ram::<N>::from(&arr);
        assert_eq!(*ram.0, arr);

        let vec: Vec<u8> = (0..N).map(|x| x as u8).collect();
        let buf = vec.try_into().unwrap();
        let ram = Ram::<N>::from(&buf);
        assert_eq!(*ram.0, buf);
    }

    #[test]
    fn address_read_write_works() {
        let mut ram = Ram::<0x1>::new();
        assert_eq!(ram.read(0x0), 0x00);
        ram.write(0x0, 0xaa);
        assert_eq!(ram.read(0x0), 0xaa);
    }

    #[test]
    fn device_contains_works() {
        const N0: usize = 0x0;
        let ram = Ram::<N0>::new();
        (0..N0).for_each(|addr| assert!(ram.contains(addr)));

        const N1: usize = 0x1;
        let ram = Ram::<N1>::new();
        (0..N1).for_each(|addr| assert!(ram.contains(addr)));

        const N2: usize = 0x10;
        let ram = Ram::<N2>::new();
        (0..N2).for_each(|addr| assert!(ram.contains(addr)));

        const N3: usize = 0x100;
        let ram = Ram::<N3>::new();
        (0..N3).for_each(|addr| assert!(ram.contains(addr)));

        const N4: usize = 0x1000;
        let ram = Ram::<N4>::new();
        (0..N4).for_each(|addr| assert!(ram.contains(addr)));

        const N5: usize = 0x10000;
        let ram = Ram::<N5>::new();
        (0..N5).for_each(|addr| assert!(ram.contains(addr)));
    }
}
