use super::Device;

/// Null device.
///
/// # Usage
///
/// The [`Null`] device ignores all writes, and always yields the same "garbage"
/// values when read. This can be useful to allow memory accesses to an unmapped
/// region of memory without causing a panic.
///
/// [`Null`] defaults to yielding `0x00` when read, but this can be changed
/// either by initializing with [`Null::with()`], or through the
/// [`Null::read_as()`] method at runtime.
#[derive(Debug, Default)]
pub struct Null<const N: usize>(u8);

impl<const N: usize> Null<N> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(read_as: u8) -> Self {
        Self(read_as)
    }

    pub fn read_as(&mut self, byte: u8) {
        self.0 = byte;
    }
}

impl<const N: usize> Device for Null<N> {
    fn len(&self) -> usize {
        N
    }

    fn read(&self, _index: usize) -> u8 {
        self.0
    }

    fn write(&mut self, _index: usize, _value: u8) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_works() {
        assert_eq!(std::mem::size_of::<Null::<0x0>>(), 1);
        assert_eq!(std::mem::size_of::<Null::<0x1>>(), 1);
        assert_eq!(std::mem::size_of::<Null::<0x10>>(), 1);
        assert_eq!(std::mem::size_of::<Null::<0x100>>(), 1);
        assert_eq!(std::mem::size_of::<Null::<0x1000>>(), 1);
        assert_eq!(std::mem::size_of::<Null::<0x10000>>(), 1);
    }

    #[test]
    fn new_works() {
        let null = Null::<0x100>::new();
        assert!((0x000..0x100)
            .map(|addr| null.read(addr))
            .all(|byte| byte == 0));
    }

    #[test]
    fn with_works() {
        let null = Null::<0x100>::with(0xaa);
        assert!((0x000..0x100)
            .map(|addr| null.read(addr))
            .all(|byte| byte == 0xaa));
    }

    #[test]
    fn device_len_works() {
        const N0: usize = 0x0;
        let null = Null::<N0>::new();
        assert_eq!(null.len(), N0);

        const N1: usize = 0x1;
        let null = Null::<N1>::new();
        assert_eq!(null.len(), N1);

        const N2: usize = 0x10;
        let null = Null::<N2>::new();
        assert_eq!(null.len(), N2);

        const N3: usize = 0x100;
        let null = Null::<N3>::new();
        assert_eq!(null.len(), N3);

        const N4: usize = 0x1000;
        let null = Null::<N4>::new();
        assert_eq!(null.len(), N4);

        const N5: usize = 0x10000;
        let null = Null::<N5>::new();
        assert_eq!(null.len(), N5);
    }

    #[test]
    fn device_read_works() {
        let null = Null::<0x100>::with(0xaa);
        assert!((0x000..0x100)
            .map(|addr| null.read(addr))
            .all(|byte| byte == null.0));
    }

    #[test]
    fn device_write_works() {
        let mut null = Null::<0x100>::new();
        (0x000..0x100).for_each(|addr| null.write(addr, 0xaa));
        assert!((0x000..0x100)
            .map(|addr| null.read(addr))
            .all(|byte| byte == 0));
    }
}
