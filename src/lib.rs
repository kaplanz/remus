//! # Rust Emulation Suite
//! > A modular emulator creation toolkit.
//!
//! Remus provides the basic primitives for the creation of emulators. These
//! building blocks can be remixed to emulate a variety of systems.
//!
//! # Examples
//!
//! For an example of how to use Remus, consult
//! <https://github.com/kaplanz/gameboy>.

#![warn(clippy::pedantic)]

mod arc;
mod blk;
mod clk;
mod fsm;
mod pcb;

pub mod bus;
pub mod dev;
pub mod mem;
pub mod reg;

pub use self::arc::{Address, Location};
pub use self::blk::Block;
pub use self::clk::Clock;
#[doc(inline)]
pub use self::dev::{Device, Dynamic, Shared};
pub use self::fsm::Machine;
#[doc(inline)]
pub use self::pcb::Board;
