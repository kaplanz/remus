use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Memory<const N: usize>([u8; N]);

impl<const N: usize> Memory<N> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<const N: usize> Default for Memory<N> {
    fn default() -> Self {
        Self([Default::default(); N])
    }
}

impl<const N: usize> Deref for Memory<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for Memory<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> Display for Memory<N> {
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

impl<const N: usize> From<&[u8; N]> for Memory<N> {
    fn from(arr: &[u8; N]) -> Self {
        Self(*arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_works() {
        assert_eq!(std::mem::size_of::<Memory::<0x0>>(), 0x0);
        assert_eq!(std::mem::size_of::<Memory::<0x1>>(), 0x1);
        assert_eq!(std::mem::size_of::<Memory::<0x10>>(), 0x10);
        assert_eq!(std::mem::size_of::<Memory::<0x100>>(), 0x100);
        assert_eq!(std::mem::size_of::<Memory::<0x1000>>(), 0x1000);
        assert_eq!(std::mem::size_of::<Memory::<0x10000>>(), 0x10000);
    }

    #[test]
    fn new_works() {
        let ram = Memory::<0x100>::new();
        assert!(ram.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_works() {
        const N: usize = 0x100;

        let arr = [0; N];
        let ram = Memory::<N>::from(&arr);
        assert_eq!(*ram, arr);

        let vec: Vec<u8> = (0..N).map(|x| x as u8).collect();
        let buf = vec.try_into().unwrap();
        let ram = Memory::<N>::from(&buf);
        assert_eq!(*ram, buf);
    }

    #[test]
    fn device_len_works() {
        const N0: usize = 0x0;
        let ram = Memory::<N0>::new();
        assert_eq!(ram.len(), N0);

        const N1: usize = 0x1;
        let ram = Memory::<N1>::new();
        assert_eq!(ram.len(), N1);

        const N2: usize = 0x10;
        let ram = Memory::<N2>::new();
        assert_eq!(ram.len(), N2);

        const N3: usize = 0x100;
        let ram = Memory::<N3>::new();
        assert_eq!(ram.len(), N3);

        const N4: usize = 0x1000;
        let ram = Memory::<N4>::new();
        assert_eq!(ram.len(), N4);

        const N5: usize = 0x10000;
        let ram = Memory::<N5>::new();
        assert_eq!(ram.len(), N5);
    }

    #[test]
    fn display_works() {
        const N: usize = 0x100;

        let mut arr = [0; N];
        (0..0x10).for_each(|i| {
            arr[i] = i as u8;
            arr[N - i - 1] = i as u8;
        });
        let ram = Memory::<N>::from(&arr);

        assert_eq!(
            format!("{ram}"),
            [
                r"0x000: 0001 0203 0405 0607 0809 0a0b 0c0d 0e0f",
                r".....: 0000 0000 0000 0000 0000 0000 0000 0000",
                r"0x0f0: 0f0e 0d0c 0b0a 0908 0706 0504 0302 0100",
            ]
            .join("\n")
        );
    }
}
