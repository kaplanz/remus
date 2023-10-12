use thiserror::Error;

use crate::arch::{Address, TryAddress, Value};
use crate::blk::Block;
use crate::dev::Device;

/// Read-only memory model.
///
/// # Panics
///
/// Panics on [`Address::write`].
#[derive(Debug)]
pub struct Rom<V, const N: usize>(Box<[V; N]>)
where
    V: Value;

impl<V, const N: usize> Rom<V, N>
where
    V: Value,
{
    /// Constructs a new, empty `Rom<N>`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Idx, V, const N: usize> Address<Idx, V> for Rom<V, N>
where
    Idx: Value,
    V: Value,
    usize: From<Idx>,
{
    fn read(&self, index: Idx) -> V {
        self.try_read(index).unwrap()
    }

    /// # Panics
    ///
    /// Panics when attempting to write to a [`Rom`].
    fn write(&mut self, index: Idx, value: V) {
        let err = self.try_write(index, value).unwrap_err();
        panic!("`<Rom as Address>::write`: {err}");
    }
}

impl<Idx, V, const N: usize> TryAddress<Idx, V> for Rom<V, N>
where
    Idx: Value,
    V: Value,
    usize: From<Idx>,
{
    type Error = Error<Idx>;

    fn try_read(&self, index: Idx) -> Result<V, Self::Error> {
        self.0
            .get(usize::from(index))
            .copied()
            .ok_or(Error::Bounds(index))
    }

    fn try_write(&mut self, index: Idx, _: V) -> Result<(), Self::Error> {
        match self.0.get(usize::from(index)) {
            Some(_) => Err(Error::Write),
            None => Err(Error::Bounds(index)),
        }
    }
}

impl<V, const N: usize> Block for Rom<V, N> where V: Value {}

impl<V, const N: usize> Default for Rom<V, N>
where
    V: Value,
{
    fn default() -> Self {
        Self(
            vec![Default::default(); N]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        )
    }
}

impl<Idx, V, const N: usize> Device<Idx, V> for Rom<V, N>
where
    Idx: Value,
    V: Value,
    usize: From<Idx>,
{
}

impl<V, const N: usize> From<&[V; N]> for Rom<V, N>
where
    V: Value,
{
    fn from(arr: &[V; N]) -> Self {
        Self(Vec::from(&arr[..]).into_boxed_slice().try_into().unwrap())
    }
}

/// A type specifying general categories of [`Rom`] error.
#[derive(Debug, Error)]
pub enum Error<Idx: Value> {
    #[error("index out of bounds: {0:?}")]
    Bounds(Idx),
    #[error("unsupported operation: write")]
    Write,
}

#[allow(clippy::cast_possible_truncation)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let rom = Rom::<u8, 0x100>::new();
        assert!(rom.0.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_works() {
        const N: usize = 0x100;

        let arr = [0; N];
        let rom = Rom::from(&arr);
        assert_eq!(*rom.0, arr);

        let vec: Vec<u8> = (0..N).map(|x| x as u8).collect();
        let buf: [u8; N] = vec.try_into().unwrap();
        let rom = Rom::from(&buf);
        assert_eq!(*rom.0, buf);
    }

    #[test]
    fn address_read_works() {
        let rom = Rom::from(&[0xaa]);
        assert_eq!(rom.read(0x0usize), 0xaa);
    }

    #[test]
    #[should_panic]
    fn address_write_panics() {
        let mut rom = Rom::from(&[0xaa]);
        rom.write(0x0usize, 0xaa);
    }
}
