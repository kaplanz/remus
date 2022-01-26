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

pub use self::ram::Ram;
pub use self::rom::Rom;

mod ram {
    use std::fmt::{Debug, Display};
    use std::ops::{Deref, DerefMut};

    /// Random-access memory model.
    #[derive(Debug)]
    pub struct Ram<const N: usize>([u8; N]);

    impl<const N: usize> Ram<N> {
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl<const N: usize> Default for Ram<N> {
        fn default() -> Self {
            Self([Default::default(); N])
        }
    }

    impl<const N: usize> Deref for Ram<N> {
        type Target = [u8];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<const N: usize> DerefMut for Ram<N> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<const N: usize> Display for Ram<N> {
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

    impl<const N: usize> From<&[u8; N]> for Ram<N> {
        fn from(arr: &[u8; N]) -> Self {
            Self(*arr)
        }
    }
}

mod rom {
    use std::fmt::{Debug, Display};
    use std::ops::Deref;

    use crate::dev::Device;

    /// Read-only memory model.
    #[derive(Debug)]
    pub struct Rom<const N: usize>([u8; N]);

    impl<const N: usize> Rom<N> {
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl<const N: usize> Default for Rom<N> {
        fn default() -> Self {
            Self([Default::default(); N])
        }
    }

    impl<const N: usize> Deref for Rom<N> {
        type Target = [u8];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<const N: usize> Device for Rom<N> {
        fn len(&self) -> usize {
            <[u8]>::len(self)
        }

        fn read(&self, index: usize) -> u8 {
            self[index]
        }

        /// # Panics
        ///
        /// Panics when attempting to write to a [`Rom`].
        fn write(&mut self, _index: usize, _value: u8) {
            panic!("called `Device::write()` on a `Rom`");
        }
    }

    impl<const N: usize> Display for Rom<N> {
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

    impl<const N: usize> From<&[u8; N]> for Rom<N> {
        fn from(arr: &[u8; N]) -> Self {
            Self(*arr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::Device;

    #[test]
    fn size_of_works() {
        assert_eq!(std::mem::size_of::<Ram::<0x0>>(), 0x0);
        assert_eq!(std::mem::size_of::<Rom::<0x1>>(), 0x1);
        assert_eq!(std::mem::size_of::<Ram::<0x10>>(), 0x10);
        assert_eq!(std::mem::size_of::<Rom::<0x100>>(), 0x100);
        assert_eq!(std::mem::size_of::<Ram::<0x1000>>(), 0x1000);
        assert_eq!(std::mem::size_of::<Rom::<0x10000>>(), 0x10000);
    }

    #[test]
    fn new_works() {
        // Ram
        let ram = Ram::<0x100>::new();
        assert!(ram.iter().all(|&byte| byte == 0));
        // Rom
        let rom = Rom::<0x100>::new();
        assert!(rom.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_works() {
        const N: usize = 0x100;

        // Ram
        let arr = [0; N];
        let ram = Ram::<N>::from(&arr);
        assert_eq!(*ram, arr);

        let vec: Vec<u8> = (0..N).map(|x| x as u8).collect();
        let buf = vec.try_into().unwrap();
        let ram = Ram::<N>::from(&buf);
        assert_eq!(*ram, buf);

        // Rom
        let arr = [0; N];
        let rom = Rom::<N>::from(&arr);
        assert_eq!(*rom, arr);

        let vec: Vec<u8> = (0..N).map(|x| x as u8).collect();
        let buf = vec.try_into().unwrap();
        let rom = Rom::<N>::from(&buf);
        assert_eq!(*rom, buf);
    }

    #[test]
    fn device_len_works() {
        const N0: usize = 0x0;
        let ram = Ram::<N0>::new();
        assert_eq!(ram.len(), N0);

        const N1: usize = 0x1;
        let ram = Rom::<N1>::new();
        assert_eq!(ram.len(), N1);

        const N2: usize = 0x10;
        let ram = Ram::<N2>::new();
        assert_eq!(ram.len(), N2);

        const N3: usize = 0x100;
        let ram = Rom::<N3>::new();
        assert_eq!(ram.len(), N3);

        const N4: usize = 0x1000;
        let ram = Ram::<N4>::new();
        assert_eq!(ram.len(), N4);

        const N5: usize = 0x10000;
        let ram = Rom::<N5>::new();
        assert_eq!(ram.len(), N5);
    }

    #[test]
    fn ram_device_read_write_works() {
        let mut ram = Ram::<0x1>::new();
        assert_eq!(ram.read(0x0), 0x00);
        ram.write(0x0, 0xaa);
        assert_eq!(ram.read(0x0), 0xaa);
    }

    #[test]
    fn rom_device_read_works() {
        let rom = Rom::<0x1>::from(&[0xaa]);
        assert_eq!(rom.read(0x0), 0xaa);
    }

    #[test]
    #[should_panic]
    fn rom_device_write_panics() {
        let mut rom = Rom::<0x1>::from(&[0xaa]);
        rom.write(0x0, 0xaa);
    }

    #[test]
    fn display_works() {
        const N: usize = 0x100;

        let mut arr = [0; N];
        (0..0x10).for_each(|i| {
            arr[i] = i as u8;
            arr[N - i - 1] = i as u8;
        });

        // Ram
        let ram = Ram::<N>::from(&arr);
        assert_eq!(
            format!("{ram}"),
            [
                r"0x000: 0001 0203 0405 0607 0809 0a0b 0c0d 0e0f",
                r".....: 0000 0000 0000 0000 0000 0000 0000 0000",
                r"0x0f0: 0f0e 0d0c 0b0a 0908 0706 0504 0302 0100",
            ]
            .join("\n")
        );

        // Rom
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
