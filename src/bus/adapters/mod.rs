//! Device adapters.
//!
//! # Usage
//!
//! All device adapters are used as wrappers around another (shared) device.
//! Through nesting adapters, users should be able to emulate fairly complex
//! memory-mapped layouts.
//!
//! Device adapters are themselves a [`Device`](crate::dev::Device), so they use
//! the same interface as the devices they are modifying. As well, they all own
//! the devices they modify through an `Rc<RefCell<dyn Device>>`, allowing for
//! sharing and reuse elsewhere.

pub use self::bank::Bank;
pub use self::remap::Remap;
pub use self::view::View;

mod bank;
mod remap;
mod view;
