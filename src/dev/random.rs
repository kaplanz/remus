use super::Device;
use crate::arc::Address;
use crate::blk::Block;

/// Random device.
///
/// # Usage
///
/// The `Random` device ignores all writes, and always yields random "garbage"
/// values when read. This can be useful to allow memory accesses to an unmapped
/// region of memory without causing a panic.
#[derive(Debug, Default)]
pub struct Random<const N: usize>();

impl<const N: usize> Random<N> {
    /// Constructs a new `Random`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const N: usize> Address for Random<N> {
    fn read(&self, _index: usize) -> u8 {
        rand::random()
    }

    fn write(&mut self, _index: usize, _value: u8) {}
}

impl<const N: usize> Block for Random<N> {}

impl<const N: usize> Device for Random<N> {
    fn contains(&self, index: usize) -> bool {
        (0..self.len()).contains(&index)
    }

    fn len(&self) -> usize {
        N
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let _ = Random::<0x100>::new();
    }

    #[test]
    fn address_read_works() {
        let random = Random::<0x100>::new();
        (0x000..0x100).for_each(|addr| {
            let _ = random.read(addr);
        });
    }

    #[test]
    fn address_write_works() {
        let mut random = Random::<0x100>::new();
        (0x000..0x100).for_each(|addr| random.write(addr, 0xaa));
        (0x000..0x100).for_each(|addr| while random.read(addr) == 0xaa {});
    }

    #[test]
    fn device_contains_works() {
        const N0: usize = 0x0;
        let random = Random::<N0>::new();
        (0..N0).for_each(|addr| assert!(random.contains(addr)));

        const N1: usize = 0x1;
        let random = Random::<N1>::new();
        (0..N1).for_each(|addr| assert!(random.contains(addr)));

        const N2: usize = 0x10;
        let random = Random::<N2>::new();
        (0..N2).for_each(|addr| assert!(random.contains(addr)));

        const N3: usize = 0x100;
        let random = Random::<N3>::new();
        (0..N3).for_each(|addr| assert!(random.contains(addr)));

        const N4: usize = 0x1000;
        let random = Random::<N4>::new();
        (0..N4).for_each(|addr| assert!(random.contains(addr)));

        const N5: usize = 0x10000;
        let random = Random::<N5>::new();
        (0..N5).for_each(|addr| assert!(random.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        assert_eq!(Random::<0x0>::new().len(), 0);
        assert_eq!(Random::<0x1>::new().len(), 0x1);
        assert_eq!(Random::<0x10>::new().len(), 0x10);
        assert_eq!(Random::<0x100>::new().len(), 0x100);
        assert_eq!(Random::<0x1000>::new().len(), 0x1000);
        assert_eq!(Random::<0x10000>::new().len(), 0x10000);
    }
}
