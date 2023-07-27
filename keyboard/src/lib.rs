#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::ptr::{addr_of_mut, null, null_mut};
use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{Flex, Input, Pin, Pull, AnyPin};
use embassy_stm32::peripherals::{PB7, PB8, PB9};
use embassy_stm32::time::khz;


pub struct Keyboard <'d, const ROW: usize, const COL: usize, const BUTK: usize>{
    input: [Input<'d, AnyPin>; ROW],
    output: [Output<'d, AnyPin>; COL]
}
fn init_row<'d> (p: AnyPin) -> Input<'d, AnyPin>{
    into_ref!(p);
    Input::new(p, Pull::Down)
}

fn init_col<'d> (p: AnyPin) -> Output<'d, AnyPin>{
    into_ref!(p);
    Output::new(p, Level::Low, Speed::Low)
}

impl <'d, const ROW: usize, const COL: usize, const BUTK: usize> Keyboard<'d, ROW, COL, BUTK>{
    pub fn new(mut inputs: [AnyPin; ROW], mut outputs: [AnyPin; COL], for_key: [u8; BUTK]) -> Self{
        Self { input: inputs.map(init_row), output: outputs.map(init_col) }
    }

    fn read_column(&mut self, column: usize) -> [u8; ROW]{
        let mut keys: [u8; ROW] = [0; ROW];
        self.output[column].set_high();
        let mut i: usize = 0;
        while i<ROW {
            if self.input[i].is_high() { keys[i] = 1; }
            i += 1;
        }
        self.output[column].set_low();
        return keys;
    }

    /*
    0 - F1, 5 - F2, 10 - #,  15 - *,
    1 - 1,  6 - 2,  11 - 3,  16 - ^,
    2 - 4,  7 - 5,  12 - 6,  17 - v,
    3 - 7,  8 - 8,  13 - 9,  18 - Esc,
    4 - <-, 9 - 0,  14 - ->, 19 - Ent,
     */
    pub fn read_key(&mut self) -> [u8; BUTK]{
        let mut keys: [u8; BUTK] = [0; BUTK];
        let mut i: usize = 0; let mut j: usize = 0;
        while i<COL {
            let tmp: [u8; ROW] = self.read_column(i);
            j = 0;
            while j<ROW {
                keys[i*COL+j] = tmp[j];
                j += 1;
            }
            i+=1;
        }
        return keys;
    }
}
/*
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
        if self.i1.is_high(){ keys[0] = 1; }
        if self.i2.is_high(){ keys[1] = 1; }
        if self.i3.is_high(){ keys[2] = 1; }
        if self.i4.is_high(){ keys[3] = 1; }
        if self.i5.is_high(){ keys[4] = 1; }
        match column {
            0 => { self.o1.set_low(); }
            1 => { self.o2.set_low(); }
            2 => { self.o3.set_low(); }
            3 => { self.o4.set_low(); }
            _ => {}
        }
        return keys;
    }

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
}*/