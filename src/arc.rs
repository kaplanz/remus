/// Addressable read-write interface.
pub trait Address {
    /// Reads from the specified address.
    fn read(&self, addr: usize) -> u8;

    /// Writes to the specified address.
    fn write(&mut self, addr: usize, value: u8);
}

/// Processor load-store interface.
pub trait Processor<V> {
    /// Accessor for specifying registers.
    ///
    /// This should normally be implemented as an enum of reguster names.
    type Register;

    /// Loads from the specified register.
    fn load(&self, reg: Self::Register) -> V;

    /// Writes to the specified register.
    fn store(&mut self, reg: Self::Register, value: V);
}
