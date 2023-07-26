use crate::blk::Block;
use crate::bus::Bus;

/// Printed circuit board.
pub trait Board: Block {
    /// Connects this board's blocks onto the bus.
    ///
    /// This must be called at least once.
    fn connect(&self, bus: &mut Bus);

    /// Disconnects this board's blocks from the bus.
    ///
    /// # Note
    ///
    /// The provided implementation does nothing.
    fn disconnect(&self, _bus: &mut Bus) {}
}
