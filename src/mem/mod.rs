//! Basic memory models.
//!
//! # Usage
//!
//! The [`Ram`] and [`Rom`] memory models work similarly to one another, with
//! the obvious exception that [`Rom`] ignores all writes. As both implement
//! [`Deref`](std::ops::Deref) into a `[u8]`, all expected [`std::slice`]
//! functions are available.
//!
//! Additionally, both models implement [`Device`](crate::dev::Device), allowing
//! them to be mapped to another address space.

use std::fmt::Display;
use std::ops::Deref;

pub use self::ram::Ram;
pub use self::rom::Rom;

mod ram;
mod rom;

/// Generic memory model.
///
/// [`Memory`] implements [`Display`] in such a way that may be convenient for
/// implementers.
///
/// Additionally, it enforces [`Deref`] and [`Device`], allowing any other types
/// which do so to trivially implement [`Memory`].
pub trait Memory: Deref<Target = [u8]> {}

impl<T: Deref<Target = [u8]>> Memory for T {}

impl Display for dyn Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Display 2-byte (16-bit) words
        const WORDSIZE: usize = 2;
        // Display (usually) 8-word rows
        const ROWSIZE: usize = WORDSIZE * std::mem::size_of::<usize>();
        // Display addresses formatted to maximum width
        let width = format!("{:#x}", self.len()).len();
        // Determine if inside skipped zero block
        let mut skip = false;

        for (i, row) in self.chunks(ROWSIZE).enumerate() {
            // Ignore row of zeros after first
            let zero = row.iter().all(|&byte| byte == 0);
            if skip {
                if zero {
                    continue;
                } else {
                    skip = false;
                }
            }
            // Insert a newline after the previous row
            if i != 0 {
                writeln!(f)?;
            }
            // Write first row of zeros
            if zero {
                write!(f, "{}:", ".".repeat(width))?;
                skip = true;
            }
            // Write row index
            else {
                write!(f, "{:#0width$x}:", i * ROWSIZE)?;
            }
            // Write row contents
            for word in row.chunks(WORDSIZE) {
                write!(f, " ")?;
                for byte in word {
                    write!(f, "{byte:02x}")?;
                }
            }
        }

        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_works() {
        const N: usize = 0x100;

        let mut arr = [0; N];
        (0..0x10).for_each(|i| {
            arr[i] = i as u8;
            arr[N - i - 1] = i as u8;
        });

        let rom = Rom::<N>::from(&arr);
        assert_eq!(
            format!("{rom}"),
            [
                r"0x000: 0001 0203 0405 0607 0809 0a0b 0c0d 0e0f",
                r".....: 0000 0000 0000 0000 0000 0000 0000 0000",
                r"0x0f0: 0f0e 0d0c 0b0a 0908 0706 0504 0302 0100",
            ]
            .join("\n")
        );
    }
}
