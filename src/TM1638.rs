#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::arch::asm;
use cortex_m::asm::delay;
use defmt::println;
use embassy_executor::Spawner;
use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{Flex, Input, Pin, Pull};
use embassy_stm32::peripherals::{PB7, PB8, PB9};
use embassy_time::{Duration, Timer};

use {defmt_rtt as _, panic_probe as _};

fn make_map() -> [u8; 320] {
    let map: [u8; 320] = [
        1, 1, 1, 1, 1, 1, 0, 0, // 0
        0, 1, 1, 0, 0, 0, 0, 0, // 1
        1, 1, 0, 1, 1, 0, 1, 0, // 2
        1, 1, 1, 1, 0, 0, 1, 0, // 3
        0, 1, 1, 0, 0, 1, 1, 0, // 4
        1, 0, 1, 1, 0, 1, 1, 0, // 5
        1, 0, 1, 1, 1, 1, 1, 0, // 6
        1, 1, 1, 0, 0, 0, 0, 0, // 7
        1, 1, 1, 1, 1, 1, 1, 0, // 8
        1, 1, 1, 1, 0, 1, 1, 0, // 9
        1, 1, 1, 0, 1, 1, 1, 0, // A
        0, 0, 1, 1, 1, 1, 1, 0, // b
        1, 0, 0, 1, 1, 1, 0, 0, // C
        0, 1, 1, 1, 1, 0, 1, 0, // d
        1, 0, 0, 1, 1, 1, 1, 0, // E
        1, 0, 0, 0, 1, 1, 1, 0, // F
        1, 0, 1, 1, 1, 1, 0, 0, // G
        0, 0, 1, 0, 1, 1, 1, 0, // H
        0, 0, 0, 0, 1, 1, 0, 0, // I
        0, 1, 1, 1, 1, 0, 0, 0, // J
        1, 0, 1, 0, 1, 1, 1, 0, // K
        0, 0, 0, 1, 1, 1, 0, 0, // L
        1, 0, 1, 0, 1, 0, 0, 0, // M
        1, 1, 1, 0, 1, 1, 0, 0, // N
        1, 1, 1, 1, 1, 1, 0, 0, // O
        1, 1, 0, 0, 1, 1, 1, 0, // P
        1, 1, 1, 0, 0, 1, 1, 0, // Q
        1, 1, 0, 0, 1, 1, 0, 0, // R
        1, 0, 1, 1, 0, 1, 1, 0, // S
        0, 0, 0, 1, 1, 1, 1, 0, // T
        0, 1, 1, 1, 1, 1, 0, 0, // U
        0, 0, 1, 1, 1, 0, 0, 0, // V
        0, 1, 0, 1, 0, 1, 0, 0, // W
        0, 1, 1, 0, 1, 1, 1, 0, // X
        0, 1, 1, 1, 0, 1, 1, 0, // Y
        1, 1, 0, 1, 1, 0, 1, 0, // Z
        0, 0, 0, 0, 0, 0, 1, 0, // -
        0, 0, 0, 1, 0, 0, 0, 0, // _
        1, 1, 1, 1, 1, 1, 1, 1, // B.
        1, 1, 1, 1, 1, 1, 0, 1 // D.
    ];
    return map;
}

fn make_bin(num: u8) -> [u8; 8]{
    return [(num>=128) as u8, ((num%128)>=64)as u8, ((num%64)>=32)as u8, ((num%32)>=16)as u8, ((num%16)>=8)as u8, ((num%8)>=4)as u8, ((num%4)>=2) as u8, (num%2)]
}

pub(crate) struct LedAndKey<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>{
    stb1: Output<'d, STB1>,
    stb2: Output<'d, STB2>,
    clk: Output<'d, CLK>,
    dio: Flex<'d, DIO>,
    pos: u8,
    b_skin: bool,
    d_skin: bool,
    map: [u8; 320]
}

impl <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin> LedAndKey <'d, STB1, STB2, CLK, DIO> {
    pub(crate) fn new(s1:STB1, s2:STB2, c:CLK, d:DIO) -> LedAndKey<'d, STB1, STB2, CLK, DIO>{
        let mut stba = Output::new(s1, Level::Low, Speed::Low);
        let mut stbb = Output::new(s2, Level::Low, Speed::Low);
        let mut clka = Output::new(c, Level::Low, Speed::Low);
        let mut dioa = Flex::new(d);
        stba.set_high(); stbb.set_high(); clka.set_low();
        dioa.set_as_input_output(Speed::Low, Pull::Up);
        Self { stb1: stba, stb2:stbb, clk: clka, dio: dioa, pos: 0, b_skin: false, d_skin: false, map: make_map() }
    }

    pub(crate) fn send_byte(&mut self, command: [u8; 8]){
        let mut i: i32 = 7;
        while i >= 0 {
            if(command[i as usize] == 1){ self.dio.set_high(); } else { self.dio.set_low(); }
            self.clk.set_high(); self.clk.set_low();
            i-=1;
        }
    }
    pub(crate) fn turn_on_display(&mut self, brightness: u8){
        self.stb1.set_high(); self.stb2.set_high();
        self.stb1.set_low(); self.stb2.set_low();
        self.send_byte(make_bin(brightness+136));
        self.stb1.set_high(); self.stb2.set_high();
    }

    pub(crate) fn turn_off_display(&mut self){
        self.stb1.set_high(); self.stb2.set_high();
        self.stb1.set_low(); self.stb2.set_low();
        self.send_byte(make_bin(128));
        self.stb1.set_high(); self.stb2.set_high();
    }

    pub(crate) fn move_cursor(&mut self, mut position: u8){
        self.pos = position;
        if(position<16) {
            self.stb1.set_high(); self.stb1.set_low();
            self.stb2.set_high();
        }else{
            self.stb2.set_high(); self.stb2.set_low();
            self.stb1.set_high();
            position %= 16;
        }
        self.send_byte(make_bin(position+192));
    }

    pub(crate) fn clean_display(&mut self){
        self.stb1.set_low(); self.stb2.set_low();
        let mut count : u8  = 0;
        while count < 17{
            self.send_byte([0; 8]);
            count+=1;
        }
        self.stb1.set_high(); self.stb2.set_high();
    }

    pub(crate) fn swap_b_skin(&mut self){
        self.b_skin = !self.b_skin;
    }

    pub(crate) fn swap_d_skin(&mut self){
        self.d_skin = !self.d_skin;
    }

    pub(crate) fn print_char(&mut self, character: char){
        let mut count: usize = 0; let mut pos: usize = 0;
        if (character == '-'){ pos = 288; }
        else if (character == '_') { pos = 296; }
        else if (character == 'b' || character == 'B'){
            if (self.b_skin){  pos = 304; } else {
                pos = 88;
            }
        }
        else if (character == 'd' || character == 'D') {
            if(self.d_skin){ pos = 312; }
            else { pos = 104; }
        }
        else if((character as u8)<58) { pos = (character as usize - 48) * 8; }
        else if((character as u8)<97) { pos = 80 + 8 * (character as usize - 65);}
        else { pos = 80 + 8 * (character as usize - 97); }
        while(count<8){
            if(self.map[pos + count] == 1){ self.dio.set_high(); } else { self.dio.set_low(); }
            self.clk.set_high(); self.clk.set_low();
            count += 1;
        }
        self.pos += 1;
    }

    pub(crate) fn print(&mut self, s: &str){
        let mut count: u8 = 0;
        for ch in s.chars(){
            if(count==16){ break; }
            if ((ch >='0' && ch <= '9') || (ch>='a' && ch <= 'z') || (ch == '-') || (ch == '_') ){
                self.print_char(ch);
                self.send_byte([0; 8]);
                self.pos += 1; self.pos %= 32;
                count += 1;
                if (self.pos == 0) { self.move_cursor(0); }
                else if(self.pos == 16){ self.move_cursor(16); }
            }
            else if (ch >= 'A' && ch <= 'Z') {
                let new_ch: u8 = ch as u8 + 32;
                self.print_char(new_ch as char);
                self.send_byte([0; 8]);
                self.pos += 1; self.pos %= 32;
                count += 1;
                if (self.pos == 0) { self.move_cursor(0); }
                else if(self.pos == 16){ self.move_cursor(16); }
            }
        }
    }

    fn read_command(&mut self){
        let mut count: usize = 0;
        self.dio.set_low(); self.clk.set_high(); self.clk.set_low();
        self.dio.set_high(); self.clk.set_high(); self.clk.set_low();
        self.dio.set_low();
        while (count<4){
            self.clk.set_high(); self.clk.set_low();
            count += 1;
        }
        self.dio.set_high(); self.clk.set_high(); self.clk.set_low();
        self.dio.set_low(); self.clk.set_high();
    }

    pub(crate) fn read_key(&mut self) -> [u8; 16]{
        self.stb1.set_high(); self.stb1.set_low();
        self.stb2.set_high();
        self.read_command();
        let mut keys: [u8; 16] = [0; 16];
        let mut count: usize = 0;
        self.dio.set_high();
        while (count<32) { // 1 - 0, 2 - 8, 3 - 16, 4 - 24, 5 - 4, 6 - 12, 7 - 20, 8 - 28
            self.clk.set_low();
            if(self.dio.is_high()) {
                if (count % 8 == 0) { keys[count / 8] = 1; }
                else if (count % 8 == 4) { keys[4 + ((count - 4) / 8)] = 1 }
            }
            if(count != 31){ self.clk.set_high(); }
            count+=1;
        }
        self.stb1.set_high(); self.stb2.set_low();
        self.read_command();
        self.dio.set_high();
        count = 0;
        while (count<32) { // 1 - 0, 2 - 8, 3 - 16, 4 - 24, 5 - 4, 6 - 12, 7 - 20, 8 - 28
            self.clk.set_low();
            if(self.dio.is_high()) {
                if (count % 8 == 0) { keys[8 + (count / 8)] = 1; }
                else if (count % 8 == 4) { keys[12 + ((count - 4) / 8)] = 1; }
            }
            if(count != 31){ self.clk.set_high(); }
            count+=1;
        }
        self.stb2.set_high();
        return keys;
    }
}

