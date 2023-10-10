//! Basic register models.
//!
//! # Usage
//!
//! The [`Register`] model should be used as a quick memory cell. It is generic
//! over the unsigned integer types, representing registers holding values of
//! types [`u8`], [`u16`], [`u32`], [`u64`], and [`u128`] respectively.
//!
//! To provide access as the represented type, `Register` implements [`Cell`].
//!
//! Since `Register` implements [`Device`], it may be mapped to another address
//! space using a [`Bus`](crate::bus::Bus), and is [byte-addressable] through
//! [`Address::read`] and [`Address::write`].
//!
//! [newtype pattern]:  https://doc.rust-lang.org/rust-by-example/generics/new_types.html
//! [byte-addressable]: https://en.wikipedia.org/wiki/Byte_addressing

use std::default::Default;
use std::fmt::Debug;

use crate::arch::{Address, Cell, Value};
use crate::blk::Block;
use crate::dev::Device;

/// Register model.
#[derive(Debug, Default)]
pub struct Register<V>(V)
where
    V: Value;

impl<V> Register<V>
where
    V: Value,
{
    /// Constructs a new `Register<U>`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Idx, V> Address<Idx, V> for Register<V>
where
    Idx: Value,
    V: Value,
{
    fn read(&self, _: Idx) -> V {
        self.load()
    }

    fn write(&mut self, _: Idx, value: V) {
        self.store(value);
    }
}

impl<V> Cell<V> for Register<V>
where
    V: Value,
{
    fn load(&self) -> V {
        self.0
    }

    fn store(&mut self, value: V) {
        self.0 = value;
    }
}

impl<V> Block for Register<V>
where
    V: Value,
{
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl<V> From<V> for Register<V>
where
    V: Value,
{
    fn from(value: V) -> Self {
        Self(value)
    }
}

impl<Idx, V> Device<Idx, V> for Register<V>
where
    Idx: Value,
    V: Value,
    Register<V>: Address<Idx, V>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        // 8-bit register
        let r8 = Register::<u8>::new();
        assert_eq!(r8.0, 0_u8);

        // 16-bit register
        let r16 = Register::<u16>::new();
        assert_eq!(r16.0, 0_u16);

        // 32-bit register
        let r32 = Register::<u32>::new();
        assert_eq!(r32.0, 0_u32);

        // 64-bit register
        let r64 = Register::<u64>::new();
        assert_eq!(r64.0, 0_u64);

        // 128-bit register
        let r128 = Register::<u128>::new();
        assert_eq!(r128.0, 0_u128);
    }

    #[test]
    fn from_works() {
        // 8-bit register
        let r8 = Register::<u8>::from(0x01_u8);
        assert_eq!(r8.0, 0x01_u8);

        // 16-bit register
        let r16 = Register::<u16>::from(0x0123_u16);
        assert_eq!(r16.0, 0x0123_u16);

        // 32-bit register
        let r32 = Register::<u32>::from(0x0123_4567_u32);
        assert_eq!(r32.0, 0x0123_4567_u32);

        // 64-bit register
        let r64 = Register::<u64>::from(0x0123_4567_89ab_cdef_u64);
        assert_eq!(r64.0, 0x0123_4567_89ab_cdef_u64);

        // 128-bit register
        let r128 = Register::<u128>::from(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef_u128);
        assert_eq!(r128.0, 0x0123_4567_89ab_cdef_0123_4567_89ab_cdef_u128);
    }

    #[test]
    fn address_read_works() {
        // 8-bit register
        let r8 = Register::<u8>::from(0x01);
        assert_eq!(r8.read(0), 0x01);

        // 16-bit register
        let r16 = Register::<u16>::from(0x0123);
        assert_eq!(r16.read(0), 0x0123);

        // 32-bit register
        let r32 = Register::<u32>::from(0x0123_4567);
        assert_eq!(r32.read(0), 0x0123_4567);

        // 64-bit register
        let r64 = Register::<u64>::from(0x0123_4567_89ab_cdef);
        assert_eq!(r64.read(0), 0x0123_4567_89ab_cdef);

        // 128-bit register
        let r128 = Register::<u128>::from(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);
        assert_eq!(r128.read(0), 0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);
    }

    #[test]
    fn address_write_works() {
        // 8-bit register
        let mut r8 = Register::<u8>::new();
        r8.write(0, 0xaa);
        assert_eq!(r8.load(), 0xaa);

        // 16-bit register
        let mut r16 = Register::<u16>::new();
        r16.write(1, 0xbb);
        assert_eq!(r16.load(), 0xbb);

        // 32-bit register
        let mut r32 = Register::<u32>::new();
        r32.write(2, 0xcc);
        assert_eq!(r32.load(), 0xcc);

        // 64-bit register
        let mut r64 = Register::<u64>::new();
        r64.write(4, 0xdd);
        assert_eq!(r64.load(), 0xdd);

        // 128-bit register
        let mut r128 = Register::<u128>::new();
        r128.write(8, 0xee);
        assert_eq!(r128.load(), 0xee);
    }

    #[test]
    fn cell_load_works() {
        // 8-bit register
        let r8 = Register::<u8>::from(0x01_u8);
        assert_eq!(r8.load(), 0x01_u8);

        // 16-bit register
        let r16 = Register::<u16>::from(0x0123_u16);
        assert_eq!(r16.load(), 0x0123_u16);

        // 32-bit register
        let r32 = Register::<u32>::from(0x0123_4567_u32);
        assert_eq!(r32.load(), 0x0123_4567_u32);

        // 64-bit register
        let r64 = Register::<u64>::from(0x0123_4567_89ab_cdef_u64);
        assert_eq!(r64.load(), 0x0123_4567_89ab_cdef_u64);

        // 128-bit register
        let r128 = Register::<u128>::from(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef_u128);
        assert_eq!(r128.load(), 0x0123_4567_89ab_cdef_0123_4567_89ab_cdef_u128);
    }

    #[test]
    fn cell_store_works() {
        // 8-bit register
        let mut r8 = Register::<u8>::new();
        r8.store(0x01_u8);
        assert_eq!(r8.load(), 0x01_u8);

        // 16-bit register
        let mut r16 = Register::<u16>::new();
        r16.store(0x0123_u16);
        assert_eq!(r16.load(), 0x0123_u16);

        // 32-bit register
        let mut r32 = Register::<u32>::new();
        r32.store(0x0123_4567_u32);
        assert_eq!(r32.load(), 0x0123_4567_u32);

        // 64-bit register
        let mut r64 = Register::<u64>::new();
        r64.store(0x0123_4567_89ab_cdef_u64);
        assert_eq!(r64.load(), 0x0123_4567_89ab_cdef_u64);

        // 128-bit register
        let mut r128 = Register::<u128>::new();
        r128.store(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef_u128);
        assert_eq!(r128.load(), 0x0123_4567_89ab_cdef_0123_4567_89ab_cdef_u128);
    }
}
