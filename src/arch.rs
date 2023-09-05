/// Addressable read-write interface.
pub trait Address<V>
where
    V: Copy + Default,
{
    /// Reads from the specified address.
    fn read(&self, index: usize) -> V;

    /// Writes to the specified address.
    fn write(&mut self, index: usize, value: V);
}

/// Register load-store interface.
pub trait Cell<V>
where
    V: Copy + Default,
{
    /// Loads the register's value.
    fn load(&self) -> V;

    /// Stores the value into the register.
    fn store(&mut self, value: V);
}

/// Processor load-store interface.
pub trait Location<V>
where
    V: Copy + Default,
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
