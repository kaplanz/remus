use crate::blk::Block;

/// Finite-state machine.
pub trait Machine: Block {
    /// Check if the [`Machine`] is in a runnable state.
    fn enabled(&self) -> bool;

    /// Perform a single cycle of the [`Machine`]'s execution, potentially
    /// changing its state.
    fn cycle(&mut self);
}
