use std::marker::PhantomData;

use rand::distributions::Standard;
use rand::prelude::Distribution;
use thiserror::Error;

use super::Device;
use crate::arch::{Address, TryAddress, Value};
use crate::blk::Block;

/// Random device.
///
/// # Usage
///
/// The `Random` device ignores all writes, and always yields random "garbage"
/// values when read. This can be useful to allow memory accesses to an unmapped
/// region of memory without causing a panic.
#[derive(Debug, Default)]
pub struct Random<V, const N: usize = 0>(PhantomData<V>)
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

impl<Idx, V, const N: usize> TryAddress<Idx, V> for Random<V, N>
where
    Idx: Value,
    V: Value,
    usize: From<Idx>,
    Standard: Distribution<V>,
{
    type Error = Error<Idx>;

    fn try_read(&self, index: Idx) -> Result<V, Self::Error> {
        match N {
            len @ 0 | len if len > usize::from(index) => Ok(rand::random()),
            _ => Err(Error::Bounds(index)),
        }
    }

    fn try_write(&mut self, index: Idx, _: V) -> Result<(), Self::Error> {
        match N {
            len @ 0 | len if len > usize::from(index) => Ok(()),
            _ => Err(Error::Bounds(index)),
        }
    }
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

/// A type specifying general categories of [`Random`] error.
#[derive(Debug, Error)]
pub enum Error<Idx: Value> {
    #[error("index out of bounds: {0:?}")]
    Bounds(Idx),
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
