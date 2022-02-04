/// Integrated circuit block.
pub trait Block {
    /// Perform a reset on this [`Block`].
    ///
    /// Afterwards, the block should behave as if it has been re-initialized.
    ///
    /// NOTE: Models should be aware that sometimes persistent data is left
    ///       behind intentionally by the trait implementer. Within the context
    ///       of the emulator, accessing data after a reset may be considered
    ///       undefined behaviour.
    fn reset(&mut self) {}
}
