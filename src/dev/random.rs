use super::Device;

/// Random device.
///
/// # Usage
///
/// The [`Random`] device ignores all writes, and always yields random "garbage"
/// values when read. This can be useful to allow memory accesses to an unmapped
/// region of memory without causing a panic.
#[derive(Debug, Default)]
pub struct Random<const N: usize>();

impl<const N: usize> Random<N> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const N: usize> Device for Random<N> {
    fn len(&self) -> usize {
        N
    }

    fn read(&self, _index: usize) -> u8 {
        rand::random()
    }

    fn write(&mut self, _index: usize, _value: u8) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_works() {
        assert_eq!(std::mem::size_of::<Random::<0x0>>(), 0);
        assert_eq!(std::mem::size_of::<Random::<0x1>>(), 0);
        assert_eq!(std::mem::size_of::<Random::<0x10>>(), 0);
        assert_eq!(std::mem::size_of::<Random::<0x100>>(), 0);
        assert_eq!(std::mem::size_of::<Random::<0x1000>>(), 0);
        assert_eq!(std::mem::size_of::<Random::<0x10000>>(), 0);
    }

    #[test]
    fn new_works() {
        let _ = Random::<0x100>::new();
    }

    #[test]
    fn device_len_works() {
        const N0: usize = 0x0;
        let random = Random::<N0>::new();
        assert_eq!(random.len(), N0);

        const N1: usize = 0x1;
        let random = Random::<N1>::new();
        assert_eq!(random.len(), N1);

        const N2: usize = 0x10;
        let random = Random::<N2>::new();
        assert_eq!(random.len(), N2);

        const N3: usize = 0x100;
        let random = Random::<N3>::new();
        assert_eq!(random.len(), N3);

        const N4: usize = 0x1000;
        let random = Random::<N4>::new();
        assert_eq!(random.len(), N4);

        const N5: usize = 0x10000;
        let random = Random::<N5>::new();
        assert_eq!(random.len(), N5);
    }

    #[test]
    fn device_read_works() {
        let random = Random::<0x100>::new();
        (0x000..0x100).for_each(|addr| {
            let _ = random.read(addr);
        });
    }

    #[test]
    fn device_write_works() {
        let mut random = Random::<0x100>::new();
        (0x000..0x100).for_each(|addr| random.write(addr, 0xaa));
        (0x000..0x100).for_each(|addr| while random.read(addr) == 0xaa {});
    }
}
