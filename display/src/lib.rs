#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::export::display;
use defmt::println;
use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{Flex, Input, Pin, Pull};
use embassy_stm32::peripherals::{PB7, PB8, PB9};
use embassy_stm32::time::khz;
use embassy_time::{Duration, Timer};
const CHARACTERS_ON_DISPLAY: [[u8; 8]; 40] = [
        [1, 1, 1, 1, 1, 1, 0, 0], // 0
        [0, 1, 1, 0, 0, 0, 0, 0], // 1
        [1, 1, 0, 1, 1, 0, 1, 0], // 2
        [1, 1, 1, 1, 0, 0, 1, 0], // 3
        [0, 1, 1, 0, 0, 1, 1, 0], // 4
        [1, 0, 1, 1, 0, 1, 1, 0], // 5
        [1, 0, 1, 1, 1, 1, 1, 0], // 6
        [1, 1, 1, 0, 0, 0, 0, 0], // 7
        [1, 1, 1, 1, 1, 1, 1, 0], // 8
        [1, 1, 1, 1, 0, 1, 1, 0], // 9
        [1, 1, 1, 0, 1, 1, 1, 0], // A
        [0, 0, 1, 1, 1, 1, 1, 0], // b
        [1, 0, 0, 1, 1, 1, 0, 0], // C
        [0, 1, 1, 1, 1, 0, 1, 0], // d
        [1, 0, 0, 1, 1, 1, 1, 0], // E
        [1, 0, 0, 0, 1, 1, 1, 0], // F
        [1, 0, 1, 1, 1, 1, 0, 0], // G
        [0, 0, 1, 0, 1, 1, 1, 0], // H
        [0, 0, 0, 0, 1, 1, 0, 0], // I
        [0, 1, 1, 1, 1, 0, 0, 0], // J
        [1, 0, 1, 0, 1, 1, 1, 0], // K
        [0, 0, 0, 1, 1, 1, 0, 0], // L
        [1, 0, 1, 0, 1, 0, 0, 0], // M
        [1, 1, 1, 0, 1, 1, 0, 0], // N
        [1, 1, 1, 1, 1, 1, 0, 0], // O
        [1, 1, 0, 0, 1, 1, 1, 0], // P
        [1, 1, 1, 0, 0, 1, 1, 0], // Q
        [1, 1, 0, 0, 1, 1, 0, 0], // R
        [1, 0, 1, 1, 0, 1, 1, 0], // S
        [0, 0, 0, 1, 1, 1, 1, 0], // T
        [0, 1, 1, 1, 1, 1, 0, 0], // U
        [0, 0, 1, 1, 1, 0, 0, 0], // V
        [0, 1, 0, 1, 0, 1, 0, 0], // W
        [0, 1, 1, 0, 1, 1, 1, 0], // X
        [0, 1, 1, 1, 0, 1, 1, 0], // Y
        [1, 1, 0, 1, 1, 0, 1, 0], // Z
        [0, 0, 0, 0, 0, 0, 1, 0], // -
        [0, 0, 0, 1, 0, 0, 0, 0], // _
        [1, 1, 1, 1, 1, 1, 1, 1], // B.
        [1, 1, 1, 1, 1, 1, 0, 1] // D.
    ];

#[derive(Clone, Copy)]
struct KeyValue {
    key: char,
    value: [u8; 8]
}

struct Map {
    map: [KeyValue; 40]
}

impl Map {
    fn new() -> Self{
        let s: &str = "0123456789abcdefghijklmnopqrstuvwxyz-_BD";
        let mut i: usize = 0;
        let mut res: [KeyValue; 40] = [KeyValue{key: '0', value: [0; 8]}; 40];
        for c in s.chars(){
            res[i] = KeyValue{ key: c, value:CHARACTERS_ON_DISPLAY[i]};
            i += 1;
        }
        Self {map: res}
    }

    fn get(&mut self, character: char) -> [u8; 8]{
        match character {
            '-' => { return self.map[36].value }
            '_' => { return self.map[37].value }
            'B' => { return self.map[38].value }
            'D' => { return self.map[39].value }
            c if c>='0' && c<='9' => { return self.map[c as usize-'0' as usize].value }
            c if c>='a' && c<='z' => { return self.map[10+c as usize-'a' as usize].value }
            _ => {return self.map[0].value;}
        }

    }
}

fn make_bin(num: u8) -> [u8; 8]{
    return [(num>=128) as u8, ((num%128)>=64)as u8, ((num%64)>=32)as u8, ((num%32)>=16)as u8, ((num%16)>=8)as u8, ((num%8)>=4)as u8, ((num%4)>=2) as u8, (num%2)]
}

pub struct LedAndKey<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>{
    stb1: Output<'d, STB1>,
    stb2: Output<'d, STB2>,
    clk: Output<'d, CLK>,
    dio: Flex<'d, DIO>,
    pos: u8,
    b_skin: bool,
    d_skin: bool,
    map: Map
}

impl <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin> LedAndKey <'d, STB1, STB2, CLK, DIO> {
    pub fn new(s1:STB1, s2:STB2, c:CLK, d:DIO) -> LedAndKey<'d, STB1, STB2, CLK, DIO>{
        let mut stba = Output::new(s1, Level::Low, Speed::Low);
        let mut stbb = Output::new(s2, Level::Low, Speed::Low);
        let mut clka = Output::new(c, Level::Low, Speed::Low);
        let mut dioa = Flex::new(d);
        stba.set_high(); stbb.set_high(); clka.set_low();
        dioa.set_as_input_output(Speed::Low, Pull::Up);
        Self { stb1: stba, stb2:stbb, clk: clka, dio: dioa, pos: 0, b_skin: false, d_skin: false, map: Map::new() }
    }

    pub fn send_byte(&mut self, command: [u8; 8]){
        let mut i: i32 = 7;
        while i >= 0 {
            if(command[i as usize] == 1){ self.dio.set_high(); } else { self.dio.set_low(); }
            self.clk.set_high(); self.clk.set_low();
            i-=1;
        }
    }
    pub fn turn_on_display(&mut self, brightness: u8){
        self.stb1.set_high(); self.stb2.set_high();
        self.stb1.set_low(); self.stb2.set_low();
        self.send_byte(make_bin(brightness+136));
        self.stb1.set_high(); self.stb2.set_high();
    }

    pub fn turn_off_display(&mut self){
        self.stb1.set_high(); self.stb2.set_high();
        self.stb1.set_low(); self.stb2.set_low();
        self.send_byte(make_bin(128));
        self.stb1.set_high(); self.stb2.set_high();
    }

    pub fn move_cursor(&mut self, mut position: u8){
        self.pos = position;
        self.stb1.set_high(); self.stb2.set_high();
        if position<16 { self.stb1.set_low(); }
        else { self.stb2.set_low(); position %= 16; }
        self.send_byte(make_bin(position+192));
    }

    pub fn clean_display(&mut self){
        self.stb1.set_low(); self.stb2.set_low();
        let mut count : u8  = 0;
        while count < 17{
            self.send_byte([0; 8]);
            count+=1;
        }
        self.stb1.set_high(); self.stb2.set_high();
    }

    pub fn swap_b_skin(&mut self){
        self.b_skin = !self.b_skin;
    }

    pub fn swap_d_skin(&mut self){
        self.d_skin = !self.d_skin;
    }

    pub fn print_char(&mut self, character: char){
        let t = self.map.get(character);
        self.send_byte(t);
        self.pos += 1;
    }

    pub fn print(&mut self, s: &str){
        let mut count: u8 = 0;
        for ch in s.chars(){
            if count==16 { break; }
            self.pos += 1; count += 1;
            match ch {
                ch if (ch == 'b' || ch == 'B') && self.b_skin => {
                    self.print_char('B');
                    self.send_byte([0; 8]);
                }
                ch if (ch == 'd' || ch == 'D') && self.d_skin => {
                    self.print_char('D');
                    self.send_byte([0; 8]);
                }
                ch if ch>='0' && ch<='9'|| ch>='a' && ch<='z' || ch == '-' || ch == '_' => {
                    self.print_char(ch);
                    self.send_byte([0; 8]);
                }
                ch if ch >= 'A' && ch <= 'Z' => {
                    self.print_char((ch as u8 + 32) as char);
                    self.send_byte([0; 8]);
                }
                _ => { count -= 1; self.pos -= 1; }
            }
            self.pos %= 32;
            if self.pos == 0 { self.move_cursor(0); }
            else if self.pos == 16 { self.move_cursor(16); }
        }
    }

    fn read_command(&mut self){
        let mut count: usize = 0;
        self.dio.set_low(); self.clk.set_high(); self.clk.set_low();
        self.dio.set_high(); self.clk.set_high(); self.clk.set_low();
        self.dio.set_low();
        while count<4 {
            self.clk.set_high(); self.clk.set_low();
            count += 1;
        }
        self.dio.set_high(); self.clk.set_high(); self.clk.set_low();
        self.dio.set_low(); self.clk.set_high();
    }

    fn read_key(&mut self) -> [u8; 16]{
        self.stb1.set_high(); self.stb1.set_low();
        self.stb2.set_high();
        self.read_command();
        let mut keys: [u8; 16] = [0; 16];
        let mut count: usize = 0;
        self.dio.set_high();
        while count< 64 { // 1 - 0, 2 - 8, 3 - 16, 4 - 24, 5 - 4, 6 - 12, 7 - 20, 8 - 28
            self.clk.set_low();
            if self.dio.is_high() {
                match count {
                    c if c%8 == 0 && c<32 => { keys[count / 8] = 1; }
                    c if c&8 == 4 && c<32 => { keys[4 + ((count - 4) / 8)] = 1 }
                    c if c%8 == 0 && c>=32 => { keys[8 + (count / 8)] = 1; }
                    c if c&8 == 4 && c>=32 => { keys[12 + ((count - 4) / 8)] = 1; }
                    _ => {}
                }
            }
            if count%32 != 31 { self.clk.set_high(); }
            count+=1;
            if count == 32 {
                self.stb1.set_high(); self.stb2.set_low();
                self.read_command();
                self.dio.set_high();
            }
        }
        self.stb2.set_high();
        return keys;
    }
}
