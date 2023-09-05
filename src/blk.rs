use std::fmt::Debug;

use crate::share::Shared;

/// Integrated circuit block.
pub trait Block: Debug {
    /// Performs a reset on this [`Block`].
    ///
    /// Afterwards, the block should behave as if it has been
    /// re-initialized[^1].
    ///
    /// # Note
    ///
    /// The provided implementation does nothing.
    ///
    /// [^1]: Models should be aware that sometimes persistent data is left
    ///       behind intentionally by the trait implementer. Within the context
    ///       of the emulator, accessing persistent data after a reset may be
    ///       considered undefined behaviour.
    fn reset(&mut self) {}
}

/// Linked [`Block`] instance.
pub trait Linked<T: Block>: Block {
    /// Gets an owning reference to the instance.
    fn mine(&self) -> Shared<T>;

    /// Links this block's to the provided instance.
    fn link(&mut self, it: Shared<T>);
}
