pub use value::Value;

mod value {
    use std::fmt::Debug;
    use std::ops::{Add, Sub};

    /// Architecture supported integer data types.
    pub trait Value:
        Add<Output = Self> + Copy + Debug + Default + Eq + Ord + Sub<Output = Self>
    {
    }

    macro_rules! add_impl {
        ($($t:ty)*) => ($(
            impl Value for $t {}
        )*)
    }

    add_impl! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 }
}

/// Addressable read-write interface.
pub trait Address<Idx, V>
where
    Idx: Value,
    V: Value,
{
    /// Reads from the specified address.
    fn read(&self, index: Idx) -> V;

    /// Writes to the specified address.
    fn write(&mut self, index: Idx, value: V);
}

/// Register load-store interface.
pub trait Cell<V>
where
    V: Value,
{
    /// Loads the register's value.
    fn load(&self) -> V;

    /// Stores the value into the register.
    fn store(&mut self, value: V);
}

/// Processor load-store interface.
pub trait Location<V>
where
    V: Value,
{
    /// Accessor for specifying registers.
    ///
    /// This should normally be implemented as an enum of register names.
    type Register;

    /// Loads from the specified register.
    fn load(&self, reg: Self::Register) -> V;

    /// Stores to the specified register.
    fn store(&mut self, reg: Self::Register, value: V);
}
