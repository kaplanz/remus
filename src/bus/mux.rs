use super::{Error, Range};
use crate::arch::{TryAddress, Value};
use crate::dev::{Device, Dynamic};

/// Address multiplexer.
///
/// Presents the primary interface for an [address bus][Bus].
pub trait Mux<Idx, V>: Device<Idx, V> + TryAddress<Idx, V, Error = Error<Idx>>
where
    Idx: Value,
    V: Value,
{
    /// Gets the indexed device.
    fn get(&self, index: Idx) -> Option<Dynamic<Idx, V>>;

    /// Maps a device to the provided range.
    fn map(&mut self, range: Range<Idx>, dev: Dynamic<Idx, V>);

    /// Unmaps and returns a device.
    ///
    /// Returns `None` if device is not mapped.
    fn unmap(&mut self, dev: &Dynamic<Idx, V>) -> Option<Dynamic<Idx, V>>;
}
