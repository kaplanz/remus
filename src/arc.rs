/// Addressable read-write interface.
pub trait Address<V>
where
    V: Copy + Default,
{
    /// Reads from the specified address.
    fn read(&self, addr: usize) -> V;

    /// Writes to the specified address.
    fn write(&mut self, addr: usize, value: V);
}

/// Processor load-store interface.
pub trait Location<V> {
    /// Accessor for specifying registers.
    ///
    /// This should normally be implemented as an enum of register names.
    type Register;

    /// Loads from the specified register.
    fn load(&self, reg: Self::Register) -> V;

    /// Stores to the specified register.
    fn store(&mut self, reg: Self::Register, value: V);
}
