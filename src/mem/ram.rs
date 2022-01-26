use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};

use crate::mem::Memory;

/// Random-access memory model.
#[derive(Debug)]
pub struct Ram<const N: usize>([u8; N]);

impl<const N: usize> Ram<N> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<const N: usize> Default for Ram<N> {
    fn default() -> Self {
        Self([Default::default(); N])
    }
}

impl<const N: usize> Deref for Ram<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for Ram<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> Display for Ram<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self as &dyn Memory)
    }
}

impl<const N: usize> From<&[u8; N]> for Ram<N> {
    fn from(arr: &[u8; N]) -> Self {
        Self(*arr)
    }
}

impl<const N: usize> Memory for Ram<N> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::Device;

    #[test]
    fn size_of_works() {
        assert_eq!(std::mem::size_of::<Ram::<0x0>>(), 0x0);
        assert_eq!(std::mem::size_of::<Ram::<0x1>>(), 0x1);
        assert_eq!(std::mem::size_of::<Ram::<0x10>>(), 0x10);
        assert_eq!(std::mem::size_of::<Ram::<0x100>>(), 0x100);
        assert_eq!(std::mem::size_of::<Ram::<0x1000>>(), 0x1000);
        assert_eq!(std::mem::size_of::<Ram::<0x10000>>(), 0x10000);
    }

    #[test]
    fn new_works() {
        let ram = Ram::<0x100>::new();
        assert!(ram.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_works() {
        const N: usize = 0x100;

        let arr = [0; N];
        let ram = Ram::<N>::from(&arr);
        assert_eq!(*ram, arr);

        let vec: Vec<u8> = (0..N).map(|x| x as u8).collect();
        let buf = vec.try_into().unwrap();
        let ram = Ram::<N>::from(&buf);
        assert_eq!(*ram, buf);
    }

    #[test]
    fn device_len_works() {
        const N0: usize = 0x0;
        let ram = Ram::<N0>::new();
        assert_eq!(ram.len(), N0);

        const N1: usize = 0x1;
        let ram = Ram::<N1>::new();
        assert_eq!(ram.len(), N1);

        const N2: usize = 0x10;
        let ram = Ram::<N2>::new();
        assert_eq!(ram.len(), N2);

        const N3: usize = 0x100;
        let ram = Ram::<N3>::new();
        assert_eq!(ram.len(), N3);

        const N4: usize = 0x1000;
        let ram = Ram::<N4>::new();
        assert_eq!(ram.len(), N4);

        const N5: usize = 0x10000;
        let ram = Ram::<N5>::new();
        assert_eq!(ram.len(), N5);
    }

    #[test]
    fn device_read_write_works() {
        let mut ram = Ram::<0x1>::new();
        assert_eq!(ram.read(0x0), 0x00);
        ram.write(0x0, 0xaa);
        assert_eq!(ram.read(0x0), 0xaa);
    }
}
