use crate::blk::Block;
use crate::dev::{Device, SharedDevice};

/// Bank device adapter.
///
/// # Usage
///
/// The [`Bank`] device adapter provides a switchable bank of devices to be used
/// when performing [`Device`] operations.
///
/// As it is simply a wrapper, its fields are public can be accessed directly.
#[derive(Debug, Default)]
pub struct Bank {
    pub active: usize,
    pub banks: Vec<SharedDevice>,
}

impl Bank {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for Bank {
    fn reset(&mut self) {
        self.active = 0;
        for bank in &self.banks {
            bank.borrow_mut().reset();
        }
    }
}

impl Device for Bank {
    fn contains(&self, index: usize) -> bool {
        match self.banks.get(self.active) {
            Some(bank) => bank.borrow().contains(index),
            None => false,
        }
    }

    fn len(&self) -> usize {
        self.banks[self.active].borrow().len()
    }

    fn read(&self, index: usize) -> u8 {
        self.banks[self.active].borrow().read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        self.banks[self.active].borrow_mut().write(index, value);
    }
}

impl From<Vec<SharedDevice>> for Bank {
    fn from(banks: Vec<SharedDevice>) -> Self {
        Self {
            banks,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::dev::{Null, Random};
    use crate::mem::Ram;

    fn setup() -> Bank {
        let mut bank = Bank::new();
        let ram: SharedDevice = Rc::new(RefCell::new(Ram::<0x100>::from(&[0x55; 0x100])));
        let null: SharedDevice = Rc::new(RefCell::new(Null::<0>::new()));
        let random: SharedDevice = Rc::new(RefCell::new(Random::<0x100>::new()));
        bank.banks.extend([ram, null, random]);
        bank
    }

    #[test]
    fn new_works() {
        let _ = Bank::new();
    }

    #[test]
    fn device_contains_works() {
        let mut bank = setup();
        // Test bank 0
        bank.active = 0;
        (0x00..=0xff).for_each(|addr| assert!(bank.contains(addr)));
        // Test bank 1
        bank.active = 1;
        (0x00..=0xff).for_each(|addr| assert!(!bank.contains(addr)));
        // Test bank 0
        bank.active = 2;
        (0x00..=0xff).for_each(|addr| assert!(bank.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        let mut bank = setup();
        // Test bank 0
        bank.active = 0;
        assert_eq!(bank.len(), 0x100);
        // Test bank 1
        bank.active = 1;
        assert_eq!(bank.len(), 0);
        // Test bank 0
        bank.active = 2;
        assert_eq!(bank.len(), 0x100);
    }

    #[test]
    fn device_read_works() {
        let mut bank = setup();
        // Test bank 0
        bank.active = 0;
        (0x00..=0xff).for_each(|addr| assert_eq!(bank.read(addr), 0x55));
        // Test bank 2
        bank.active = 2;
        (0x00..=0xff).for_each(|addr| {
            let _ = bank.read(addr);
        });
    }

    #[test]
    fn device_write_works() {
        let mut bank = setup();
        // Test bank 0
        bank.active = 0;
        (0x00..=0xff).for_each(|addr| bank.write(addr, 0xaa));
        (0x00..=0xff).for_each(|addr| assert_eq!(bank.read(addr), 0xaa));
        // Test bank 2
        bank.active = 2;
        (0x00..=0xff).for_each(|addr| bank.write(addr, 0xaa));
        // NOTE: For all intents and purposes, this should never fail. If it
        //       does, one of two things happened:
        //       1. You broke something either in Bank or Random
        //       2. You broke this test
        //       3. You broke probability, all hope is lost
        assert!((0x00..=0xff)
            .map(|addr| bank.read(addr))
            .any(|value| value != 0xaa));
    }
}
