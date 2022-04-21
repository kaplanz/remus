use crate::blk::Block;

/// Finite-state machine.
pub trait Machine: Block {
    /// Checks if the [`Machine`] is in a runnable state.
    fn enabled(&self) -> bool;

    /// Executes a single cycle on the [`Machine`], likely mutating its state.
    fn cycle(&mut self);
}
