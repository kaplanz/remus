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

mod arch;
mod blk;
mod clk;
mod fsm;
mod pcb;
mod share;

pub mod bus;
pub mod dev;
pub mod mem;
pub mod reg;

pub use self::arch::{Address, Cell, Location};
pub use self::blk::{Block, Linked};
pub use self::clk::Clock;
pub use self::fsm::Machine;
pub use self::pcb::Board;
pub use self::share::Shared;
