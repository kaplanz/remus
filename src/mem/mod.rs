//! Basic memory models.
//!
//! # Usage
//!
//! The [`Ram`] and [`Rom`] memory models work similarly to one another, with
//! the obvious exception that `Rom` panics on writes. As both implement
//! [`Deref`](std::ops::Deref) into a `[u8]`, all expected [`std::slice`]
//! functions are available.
//!
//! Additionally, both models implement [`Device`](crate::dev::Device), allowing
//! them to be mapped to another address space.

mod ram;
mod rom;

pub use self::ram::Ram;
pub use self::rom::Rom;
