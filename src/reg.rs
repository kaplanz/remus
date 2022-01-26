use std::default::Default;
use std::ops::{Deref, DerefMut};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug)]
pub struct Register<const N: usize>([u8; N]);

impl<const N: usize> Default for Register<N> {
    fn default() -> Self {
        Self([Default::default(); N])
    }
}

impl<const N: usize> Deref for Register<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for Register<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Register<1> {
    pub fn get(&self) -> u8 {
        self.0.as_slice().read_u8().unwrap()
    }

    pub fn set(&mut self, value: u8) {
        self.0.as_mut_slice().write_u8(value).unwrap();
    }
}

impl Register<2> {
    pub fn get(&self) -> u16 {
        self.0.as_slice().read_u16::<LittleEndian>().unwrap()
    }

    pub fn set(&mut self, value: u16) {
        self.0
            .as_mut_slice()
            .write_u16::<LittleEndian>(value)
            .unwrap();
    }
}

impl Register<4> {
    pub fn get(&self) -> u32 {
        self.0.as_slice().read_u32::<LittleEndian>().unwrap()
    }

    pub fn set(&mut self, value: u32) {
        self.0
            .as_mut_slice()
            .write_u32::<LittleEndian>(value)
            .unwrap();
    }
}

impl Register<8> {
    pub fn get(&self) -> u64 {
        self.0.as_slice().read_u64::<LittleEndian>().unwrap()
    }

    pub fn set(&mut self, value: u64) {
        self.0
            .as_mut_slice()
            .write_u64::<LittleEndian>(value)
            .unwrap();
    }
}

impl Register<16> {
    pub fn get(&self) -> u128 {
        self.0.as_slice().read_u128::<LittleEndian>().unwrap()
    }

    pub fn set(&mut self, value: u128) {
        self.0
            .as_mut_slice()
            .write_u128::<LittleEndian>(value)
            .unwrap();
    }
}
