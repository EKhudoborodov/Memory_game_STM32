#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{Flex, Input, Pin, Pull};
use embassy_stm32::peripherals::{PB7, PB8, PB9};
use embassy_stm32::time::khz;
use embassy_time::{Duration, Timer};

pub struct Keyboard <'d, I1: Pin, I2: Pin, I3: Pin, I4: Pin, I5: Pin, O1: Pin, O2: Pin, O3: Pin, O4: Pin>{
    i1: Input<'d, I1>,
    i2: Input<'d, I2>,
    i3: Input<'d, I3>,
    i4: Input<'d, I4>,
    i5: Input<'d, I5>,
    o1: Output<'d, O1>,
    o2: Output<'d, O2>,
    o3: Output<'d, O3>,
    o4: Output<'d, O4>,
}

impl <'d, I1: Pin, I2: Pin, I3: Pin, I4: Pin, I5: Pin, O1: Pin, O2: Pin, O3: Pin, O4: Pin>Keyboard <'d, I1, I2, I3, I4, I5, O1, O2, O3, O4>{
    pub fn new(i1: I1, i2: I2, i3: I3, i4: I4, i5: I5, o1: O1, o2: O2, o3: O3, o4: O4) -> Keyboard<'d ,I1, I2, I3, I4, I5, O1, O2, O3, O4>{
        let mut i1 = Input::new(i1, Pull::Down);
        let mut i2 = Input::new(i2, Pull::Down);
        let mut i3 = Input::new(i3, Pull::Down);
        let mut i4 = Input::new(i4, Pull::Down);
        let mut i5 = Input::new(i5, Pull::Down);
        let mut o1 = Output::new(o1, Level::Low, Speed::Low);
        let mut o2 = Output::new(o2, Level::Low, Speed::Low);
        let mut o3 = Output::new(o3, Level::Low, Speed::Low);
        let mut o4 = Output::new(o4, Level::Low, Speed::Low);
        o1.set_low(); o2.set_low(); o3.set_low(); o4.set_low();
        Self {i1, i2, i3, i4, i5, o1, o2, o3, o4}
    }

    fn read_column(&mut self, column: u8) -> [u8; 5]{
        let mut keys: [u8; 5] = [0; 5];
        match column {
            0 => { self.o1.set_high(); }
            1 => { self.o2.set_high(); }
            2 => { self.o3.set_high(); }
            3 => { self.o4.set_high(); }
            _ => {}
        }
        if(self.i1.is_high()){ keys[0] = 1; }
        if(self.i2.is_high()){ keys[1] = 1; }
        if(self.i3.is_high()){ keys[2] = 1; }
        if(self.i4.is_high()){ keys[3] = 1; }
        if(self.i5.is_high()){ keys[4] = 1; }
        match column {
            0 => { self.o1.set_low(); }
            1 => { self.o2.set_low(); }
            2 => { self.o3.set_low(); }
            3 => { self.o4.set_low(); }
            _ => {}
        }
        return keys;
    }

    /*
    0 - F1, 5 - F2, 10 - #,  15 - *,
    1 - 1,  6 - 2,  11 - 3,  16 - ^,
    2 - 4,  7 - 5,  12 - 6,  17 - v,
    3 - 7,  8 - 8,  13 - 9,  18 - Esc,
    4 - <-, 9 - 0,  14 - ->, 19 - Ent,
     */
    pub fn read_key(&mut self) -> [u8; 20]{
        let mut keys: [u8; 20] = [0; 20];
        let mut column:u8 = 0;
        while column<4 {
            let mut count: usize = 0;
            let mut tmp: [u8; 5] = self.read_column(column);
            while count<5 {
                keys[(column as usize)*5+count] = tmp[count];
                count += 1;
            }
            column += 1;
        }
        return keys;
    }
}

/*pub fn add(left: usize, right: usize) -> usize {
    left + right
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}*/
