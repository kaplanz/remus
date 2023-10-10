use super::Device;
use crate::arch::{Address, TryAddress, Value};
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
pub struct Null<V, const N: usize = 0>(V)
where
    V: Value;

impl<V, const N: usize> Null<V, N>
where
    V: Value,
{
    /// Constructs a new `Null<N>`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct an instance of [`Null`] that yields the specified value when
    /// performing a read.
    #[must_use]
    pub fn with(value: V) -> Self {
        Self(value)
    }

    /// Set the value to be used when performing a read.
    pub fn read_as(&mut self, value: V) {
        self.0 = value;
    }
}

impl<Idx, V, const N: usize> Address<Idx, V> for Null<V, N>
where
    Idx: Value,
    V: Value,
{
    fn read(&self, _: Idx) -> V {
        self.0
    }

    fn write(&mut self, _: Idx, _: V) {}
}

impl<Idx, V, const N: usize> TryAddress<Idx, V> for Null<V, N>
where
    Idx: Value,
    V: Value,
    usize: From<Idx>,
{
    fn try_read(&self, index: Idx) -> Option<V> {
        match N {
            len @ 0 | len if len > usize::from(index) => Some(self.0),
            _ => None,
        }
    }

    fn try_write(&mut self, index: Idx, _: V) -> Option<()> {
        match N {
            len @ 0 | len if len > usize::from(index) => Some(()),
            _ => None,
        }
    }
}

impl<V, const N: usize> Block for Null<V, N> where V: Value {}

impl<Idx, V, const N: usize> Device<Idx, V> for Null<V, N>
where
    Idx: Value,
    V: Value,
{
}

#[allow(clippy::items_after_statements)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let null = Null::<u8, 0x100>::new();
        assert!((0x000..0x100)
            .map(|index| null.read(index))
            .all(|byte| byte == 0));
    }

    #[test]
    fn with_works() {
        let null: Null<u8, 0x100> = Null::with(0xaa);
        assert!((0x000..0x100)
            .map(|index| null.read(index))
            .all(|byte| byte == 0xaa));
    }

    #[test]
    fn address_read_works() {
        let null: Null<u8, 0x100> = Null::with(0xaa);
        assert!((0x000..0x100)
            .map(|index| null.read(index))
            .all(|byte| byte == null.0));
    }

    #[test]
    fn address_write_works() {
        let mut null: Null<u8, 0x100> = Null::new();
        (0x000..0x100).for_each(|index| null.write(index, 0xaa));
        assert!((0x000..0x100)
            .map(|index| null.read(index))
            .all(|byte| byte == 0));
    }
}
