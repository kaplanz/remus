use super::Device;
use crate::blk::Block;

/// Null device.
///
/// # Usage
///
/// The `Null` device ignores all writes, and always yields the same "garbage"
/// values when read. This can be useful to allow memory accesses to an unmapped
/// region of memory without causing a panic.
///
/// `Null` defaults to yielding the null byte (`0x00`) when read, but this can
/// be changed either by constructing with [`Null::with`], or through the
/// [`Null::read_as`] method at runtime.
#[derive(Debug, Default)]
pub struct Null<const N: usize>(u8);

impl<const N: usize> Null<N> {
    /// Constructs a new `Null<N>`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct an instance of [`Null`] that yields the specified byte when
    /// performing a read.
    pub fn with(read_as: u8) -> Self {
        Self(read_as)
    }

    /// Set the value to be used when performing a read.
    pub fn read_as(&mut self, byte: u8) {
        self.0 = byte;
    }
}

impl<const N: usize> Block for Null<N> {}

impl<const N: usize> Device for Null<N> {
    fn contains(&self, index: usize) -> bool {
        (0..self.len()).contains(&index)
    }

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
    fn device_contains_works() {
        const N0: usize = 0x0;
        let null = Null::<N0>::new();
        (0..N0).for_each(|addr| assert!(null.contains(addr)));

        const N1: usize = 0x1;
        let null = Null::<N1>::new();
        (0..N1).for_each(|addr| assert!(null.contains(addr)));

        const N2: usize = 0x10;
        let null = Null::<N2>::new();
        (0..N2).for_each(|addr| assert!(null.contains(addr)));

        const N3: usize = 0x100;
        let null = Null::<N3>::new();
        (0..N3).for_each(|addr| assert!(null.contains(addr)));

        const N4: usize = 0x1000;
        let null = Null::<N4>::new();
        (0..N4).for_each(|addr| assert!(null.contains(addr)));

        const N5: usize = 0x10000;
        let null = Null::<N5>::new();
        (0..N5).for_each(|addr| assert!(null.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        assert_eq!(Null::<0x0>::new().len(), 0);
        assert_eq!(Null::<0x1>::new().len(), 0x1);
        assert_eq!(Null::<0x10>::new().len(), 0x10);
        assert_eq!(Null::<0x100>::new().len(), 0x100);
        assert_eq!(Null::<0x1000>::new().len(), 0x1000);
        assert_eq!(Null::<0x10000>::new().len(), 0x10000);
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
