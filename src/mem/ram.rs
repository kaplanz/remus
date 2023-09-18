use crate::arch::{Address, Value};
use crate::blk::Block;
use crate::dev::Device;

/// Random-access memory model.
#[derive(Debug)]
pub struct Ram<V, const N: usize>(Box<[V; N]>)
where
    V: Value;

impl<V, const N: usize> Ram<V, N>
where
    V: Value,
{
    /// Constructs a new, empty `Ram<N>`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Idx, V, const N: usize> Address<Idx, V> for Ram<V, N>
where
    Idx: Value,
    V: Value,
    usize: From<Idx>,
{
    fn read(&self, index: Idx) -> V {
        self.0[usize::from(index)]
    }

    fn write(&mut self, index: Idx, value: V) {
        self.0[usize::from(index)] = value;
    }
}

impl<V, const N: usize> Block for Ram<V, N>
where
    V: Value,
{
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl<V, const N: usize> Default for Ram<V, N>
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

impl<Idx, V, const N: usize> Device<Idx, V> for Ram<V, N>
where
    Idx: Value,
    V: Value,
    usize: From<Idx>,
{
}

impl<V, const N: usize> From<&[V; N]> for Ram<V, N>
where
    V: Value,
{
    fn from(arr: &[V; N]) -> Self {
        Self(Vec::from(&arr[..]).into_boxed_slice().try_into().unwrap())
    }
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::items_after_statements)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let ram = Ram::<u8, 0x100>::new();
        assert!(ram.0.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_works() {
        const N: usize = 0x100;

        let arr = [0; N];
        let ram = Ram::from(&arr);
        assert_eq!(*ram.0, arr);

        let vec: Vec<u8> = (0..N).map(|x| x as u8).collect();
        let buf: [u8; N] = vec.try_into().unwrap();
        let ram = Ram::from(&buf);
        assert_eq!(*ram.0, buf);
    }

    #[test]
    fn address_read_write_works() {
        let mut ram: Ram<u8, 0x100> = Ram::new();
        assert_eq!(ram.read(0x0usize), 0x00);
        ram.write(0x0usize, 0xaa);
        assert_eq!(ram.read(0x0usize), 0xaa);
    }
}
