#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::export::display;
use defmt::println;
use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{AnyPin, Flex, Input, Pin, Pull};
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


struct Map <const SIZE: usize> {
    slots: [usize; 255],
    values: [[u8; 8]; SIZE],
    cur_size: usize
}

impl <const SIZE: usize>Map <SIZE> {
    fn new(s: &str) -> Self{
        let mut i: usize = 0;
        let mut slots: [usize; 255] = [0; 255];
        let mut values: [[u8; 8]; SIZE] = [[0; 8]; SIZE];
        for c in s.chars(){
            slots[(c as u8) as usize] = i;
            values[i] = CHARACTERS_ON_DISPLAY[i];
            i += 1;
        }
        Self {slots, values, cur_size: 40}
    }

    fn hash_value(&mut self, character: char) -> usize{
        return (character as u8) as usize;
    }

    fn put(&mut self, key: char, val: [u8; 8]){
        let slot = self.hash_value(key);
        self.slots[slot] = self.cur_size;
        self.values[self.cur_size] = val;
        self.cur_size += 1;
    }

    fn get(&mut self, character: char) -> [u8; 8]{
        let slot = self.hash_value(character);
        return self.values[self.slots[slot]];
    }
}

fn make_bin(num: u8) -> [u8; 8]{
    return [(num>=128) as u8, ((num%128)>=64)as u8, ((num%64)>=32)as u8, ((num%32)>=16)as u8, ((num%16)>=8)as u8, ((num%8)>=4)as u8, ((num%4)>=2) as u8, (num%2)]
}

pub struct LedAndKey<'d, const DIS: usize, const BUTD: usize, CLK: Pin, DIO: Pin, const SIZE: usize>{
    stb: [Output<'d, AnyPin>; DIS],
    clk: Output<'d, CLK>,
    dio: Flex<'d, DIO>,
    pos: usize,
    b_skin: bool,
    d_skin: bool,
    map: Map<{SIZE}>
}
fn init_stb<'d>(p: AnyPin) -> Output<'d, AnyPin>{
    into_ref!(p);
    Output::new(p, Level::High, Speed::Low)
}

impl <'d, const DIS: usize, const BUTD: usize, CLK: Pin, DIO: Pin, const SIZE: usize> LedAndKey <'d, DIS, BUTD, CLK, DIO, SIZE> {
    pub fn new(s: [AnyPin; DIS], c:CLK, d:DIO, for_game: [u8; BUTD], for_map: [u8; SIZE]) -> LedAndKey<'d, DIS, BUTD, CLK, DIO, SIZE>{
        let mut clka = Output::new(c, Level::Low, Speed::Low);
        let mut dioa = Flex::new(d);
        clka.set_low();
        dioa.set_as_input_output(Speed::Low, Pull::Up);
        Self { stb: s.map(init_stb), clk: clka, dio: dioa, pos: 0, b_skin: false, d_skin: false, map: Map::new("0123456789abcdefghijklmnopqrstuvwxyz-_BD") }
    }

    fn stb_listen_command(&mut self){
        let mut i: usize = 0;
        while i<DIS {
            self.stb[i].set_high(); self.stb[i].set_low();
            i+=1;
        }
    }

    fn mute_all(&mut self){
        let mut i: usize = 0;
        while i<DIS {
            self.stb[i].set_high();
            i+=1;
        }
    }

    fn unmute_all(&mut self){
        let mut i: usize = 0;
        while i<DIS {
            self.stb[i].set_low();
            i+=1;
        }
    }

    pub fn send_byte(&mut self, command: [u8; 8]){
        let mut i: i32 = 7;
        while i >= 0 {
            if command[i as usize] == 1 { self.dio.set_high(); } else { self.dio.set_low(); }
            self.clk.set_high(); self.clk.set_low();
            i-=1;
        }
    }
    pub fn turn_on_display(&mut self, brightness: u8){
        self.stb_listen_command();
        self.send_byte(make_bin(brightness+136));
        self.mute_all();
    }

    pub fn turn_off_display(&mut self){
        self.stb_listen_command();
        self.send_byte(make_bin(128));
        self.mute_all();
    }

    pub fn move_cursor(&mut self, mut position: usize){
        self.pos = position;
        self.mute_all();
        self.stb[(position-(position%16))/16].set_low();
        position%=16;
        self.send_byte(make_bin(position as u8+192));
    }

    pub fn clean_display(&mut self){
        self.unmute_all();
        let mut count : u8  = 0;
        while count < 17{
            self.send_byte([0; 8]);
            count+=1;
        }
        self.mute_all();
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
        let displays = DIS;
        for ch in s.chars(){
            if count== (8 * displays) as u8 { break; }
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
            self.pos %= 16*displays;
            if self.pos%16 == 0 { self.move_cursor(self.pos) }
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

    fn read_key(&mut self) -> [u8; BUTD]{
        self.mute_all();
        let mut i: usize = 0;
        let mut keys: [u8; BUTD] = [0; BUTD];
        while i<DIS {
            self.stb[i].set_low();
            self.read_command();
            let mut count: usize = 0;
            self.dio.set_high();
            while count<32 {
                self.clk.set_low();
                if self.dio.is_high() {
                    match count {
                        c if c%8 == 0 && c<32 => { keys[8*i + c / 8] = 1; }
                        c if c&8 == 4 => { keys[8*i + 4 + ((c - 4) / 8)] = 1; }
                        _ => {}
                    }
                }
                if count%32 != 31 { self.clk.set_high(); }
                count += 1;
            }
            self.stb[i].set_high();
            i+=1;
        }

        return keys;
    }
}
