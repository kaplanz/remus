use crate::blk::Block;

/// Finite-state machine.
pub trait Machine: Block {
    /// Prepare the [`Machine`] to for running, which may or may not enable it.
    ///
    /// The default implementation does nothing.
    fn setup(&mut self) {}

    /// Check if the [`Machine`] is in a runnable state.
    fn enabled(&self) -> bool;

    /// Perform a single cycle of the [`Machine`]'s execution, potentially
    /// changing its state.
    fn cycle(&mut self);
}
