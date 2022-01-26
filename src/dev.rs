use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

pub trait Device: Debug {
    fn len(&self) -> usize;

    fn read(&self, index: usize) -> u8;

    fn write(&mut self, index: usize, value: u8);

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Device for T
where
    T: Debug + Deref<Target = [u8]> + DerefMut,
{
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }

    fn read(&self, index: usize) -> u8 {
        self[index]
    }

    fn write(&mut self, index: usize, value: u8) {
        self[index] = value;
    }
}
