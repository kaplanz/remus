use std::marker::PhantomData;

use rand::distributions::Standard;
use rand::prelude::Distribution;

use super::Device;
use crate::arch::{Address, Value};
use crate::blk::Block;

/// Random device.
///
/// # Usage
///
/// The `Random` device ignores all writes, and always yields random "garbage"
/// values when read. This can be useful to allow memory accesses to an unmapped
/// region of memory without causing a panic.
#[derive(Debug, Default)]
pub struct Random<V, const N: usize>(PhantomData<V>)
where
    V: Value,
    Standard: Distribution<V>;

impl<V, const N: usize> Random<V, N>
where
    V: Value,
    Standard: Distribution<V>,
{
    /// Constructs a new `Random`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Idx, V, const N: usize> Address<Idx, V> for Random<V, N>
where
    Idx: Value,
    V: Value,
    Standard: Distribution<V>,
{
    fn read(&self, _index: Idx) -> V {
        rand::random()
    }

    fn write(&mut self, _: Idx, _: V) {}
}

impl<V, const N: usize> Block for Random<V, N>
where
    V: Value,
    Standard: Distribution<V>,
{
}

impl<Idx, V, const N: usize> Device<Idx, V> for Random<V, N>
where
    Idx: Value,
    V: Value,
    Standard: Distribution<V>,
{
}

#[allow(clippy::items_after_statements)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let _ = Random::<u8, 0x100>::new();
    }

    #[test]
    fn address_read_works() {
        let random = Random::<u8, 0x100>::new();
        (0x000..0x100).for_each(|index| {
            let _ = random.read(index);
        });
    }

    #[test]
    fn address_write_works() {
        let mut random = Random::<u8, 0x100>::new();
        (0x000..0x100).for_each(|index| random.write(index, 0xaa));
        (0x000..0x100).for_each(|index| while random.read(index) == 0xaa {});
    }
}
