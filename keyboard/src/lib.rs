#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(adt_const_params)]

use core::ptr::{addr_of_mut, null, null_mut};
use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{Flex, Input, Pin, Pull, AnyPin};
use embassy_stm32::peripherals::{PB7, PB8, PB9};
use embassy_stm32::time::khz;


struct Map <const SIZE: usize, const DIG: usize> {
    slots: [usize; 255],
    values: [bool; SIZE],
    cur_size: usize
}

impl <const SIZE: usize, const DIG: usize>Map <SIZE, DIG> {
    fn new(s: &str, digits: [char; DIG]) -> Self {
        let mut i: usize = 0;
        let mut slots: [usize; 255] = [0; 255];
        let mut values: [bool; SIZE] = [false; SIZE];
        for ch in s.chars(){
            while i<DIG {
                values[i] = (ch == digits[i]);
                i += 1;
            }
        }
        Self {slots, values, cur_size: 40}
    }

    fn hash_value(&mut self, character: char) -> usize{
        return (character as u8) as usize;
    }

    fn put(&mut self, key: char, val: bool){
        let slot = self.hash_value(key);
        self.slots[slot] = self.cur_size;
        self.values[self.cur_size] = val;
        self.cur_size += 1;
    }

    fn get(&mut self, character: char) -> bool{
        let slot = self.hash_value(character);
        return self.values[self.slots[slot]];
    }
}

pub struct Keyboard <'d, const ROW: usize, const COL: usize>{
    input: [Input<'d, AnyPin>; ROW],
    output: [Output<'d, AnyPin>; COL],
}
fn init_row<'d> (p: AnyPin) -> Input<'d, AnyPin>{
    into_ref!(p);
    Input::new(p, Pull::Down)
}

fn init_col<'d> (p: AnyPin) -> Output<'d, AnyPin>{
    into_ref!(p);
    Output::new(p, Level::Low, Speed::Low)
}

impl <'d, const ROW: usize, const COL: usize> Keyboard<'d, ROW, COL>{
    pub fn new(mut inputs: [AnyPin; ROW], mut outputs: [AnyPin; COL]) -> Self{
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
    [
    [0 - F1, 1 - F2, 2 - #,  3 - *],
    [0 - 1,  1 - 2,  2 - 3,  3 - ^],
    [0 - 4,  1 - 5,  2 - 6,  3 - v],
    [0 - 7,  1 - 8,  2 - 9,  3 - Esc],
    [0 - <-, 1 - 0,  2 - ->, 3 - Ent]
    ]
     */
    pub fn read_key(&mut self) -> [[u8; ROW]; COL]{
        let mut keys: [[u8; ROW]; COL] = [[0; ROW]; COL];
        let mut i: usize = 0; let mut j: usize = 0;
        while i<COL {
            let tmp: [u8; ROW] = self.read_column(i);
            j = 0;
            while j<ROW {
                keys[i][j] = tmp[j];
                j += 1;
            }
            i+=1;
        }
        return keys;
    }

    pub fn get_pressed(&mut self, keys: [[u8; ROW]; COL]) -> u8{
        let mut i: usize = 0; let mut j: usize = 0;
        while i<COL{
            j = 0;
            while j<ROW {
                if keys[i][j] == 1 { return (i*ROW+j+1) as u8 }
                j+=1;
            }
            i += 1;
        }
        return 0;
    }

    pub fn is_digit(&mut self, s: u8) -> bool {
        return ((s%5>1)&&(s<15)) || (s==10);
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