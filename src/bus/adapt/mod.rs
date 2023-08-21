//! Device adapters.
//!
//! # Usage
//!
//! All device adapters are used as wrappers around another (shared) device.
//! Through nesting adapters, users should be able to emulate fairly complex
//! memory-mapped layouts.
//!
//! Adapters are themselves a [`Device`](crate::dev::Device), so they use the
//! same interface as the devices they are modifying. As well, they either
//! directely own the devices they modify, or can optionally share access
//! through a [`Shared`](crate::dev::Shared) or
//! [`Dynamic`](crate::dev::Dynamic), allowing reuse elsewhere.

pub use self::bank::Bank;
pub use self::remap::Remap;
pub use self::view::View;

mod bank;
mod remap;
mod view;
