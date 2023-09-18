use crate::arch::{Address, Value};
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
        self.0[usize::from(index)]
    }

    /// # Panics
    ///
    /// Panics when attempting to write to a [`Rom`].
    fn write(&mut self, _: Idx, _value: V) {
        panic!("called `Address::write()` on a `Rom`");
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
