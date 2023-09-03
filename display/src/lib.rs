#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::export::char;
use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{AnyPin, Flex, Pin, Pull};
mod fonts;
use fonts::Symbols;
use fonts::Symbols::*;

fn make_bin(num: u8) -> [u8; 8]{
    return [(num>=128) as u8, ((num%128)>=64)as u8, ((num%64)>=32)as u8, ((num%32)>=16)as u8, ((num%16)>=8)as u8, ((num%8)>=4)as u8, ((num%4)>=2) as u8, (num%2)]
}

pub struct LedAndKey<'d, const DIS: usize, CLK: Pin, DIO: Pin>{
    stb: [Output<'d, AnyPin>; DIS],
    clk: Output<'d, CLK>,
    dio: Flex<'d, DIO>,
    pos: usize,
    b_skin: bool,
    d_skin: bool,
}
fn init_stb<'d>(p: AnyPin) -> Output<'d, AnyPin>{
    into_ref!(p);
    Output::new(p, Level::High, Speed::Low)
}

impl <'d, const DIS: usize, CLK: Pin, DIO: Pin> LedAndKey <'d, DIS, CLK, DIO> {
    pub fn new(s: [AnyPin; DIS], c:CLK, d:DIO) -> LedAndKey<'d, DIS, CLK, DIO>{
        let mut clka = Output::new(c, Level::Low, Speed::Low);
        let mut dioa = Flex::new(d);
        dioa.set_as_input_output(Speed::Low, Pull::Up);
        Self { stb: s.map(init_stb), clk: clka, dio: dioa, pos: 0, b_skin: false, d_skin: false }
    }

    fn stb_listen_command(&mut self, dis: [u8; DIS]){
        let mut i: usize = 0;
        while i<DIS {
            self.stb[i].set_high();
            if dis[i] == 1 {
                self.stb[i].set_low();
            }
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
        self.stb_listen_command([1; DIS]);
        self.send_byte(make_bin(brightness+136));
        self.stb_listen_command([0; DIS]);
    }

    pub fn turn_off_display(&mut self){
        self.stb_listen_command([1; DIS]);
        self.send_byte(make_bin(128));
        self.stb_listen_command([0; DIS]);
    }

    fn move_cursor(&mut self, mut position: usize){
        self.pos = position;
        let mut dis: [u8; DIS] = [0; DIS];
        dis[(position-(position%16))/16] = 1;
        self.stb_listen_command(dis);
        position%=16;
        self.send_byte(make_bin(position as u8+192));
    }

    pub fn clean_display(&mut self){
        self.stb_listen_command([1; DIS]);
        let mut count : u8  = 0;
        while count < 17{
            self.send_byte([0; 8]);
            count+=1;
        }
        self.stb_listen_command([0; DIS]);
    }

    pub fn swap_b_skin(&mut self){
        self.b_skin = !self.b_skin;
    }

    pub fn swap_d_skin(&mut self){
        self.d_skin = !self.d_skin;
    }

    pub fn print_char(&mut self, position: usize, character: char){
        self.move_cursor(position);
        let mut val: Symbols = match character.to_ascii_lowercase() {
            '0' => { SIM_0 }
            '1' => { SIM_1 }
            '2' => { SIM_2 }
            '3' => { SIM_3 }
            '4' => { SIM_4 }
            '5' => { SIM_5 }
            '6' => { SIM_6 }
            '7' => { SIM_7 }
            '8' => { SIM_8 }
            '9' => { SIM_9 }
            'a' => { SIM_A }
            'b' => { SIM_b }
            'c' => { SIM_C }
            'd' => { SIM_d }
            'e' => { SIM_E }
            'f' => { SIM_F }
            'g' => { SIM_G }
            'h' => { SIM_H }
            'i' => { SIM_I }
            'j' => { SIM_J }
            'k' => { SIM_K }
            'l' => { SIM_L }
            'm' => { SIM_M }
            'n' => { SIM_N }
            'o' => { SIM_0 }
            'p' => { SIM_P }
            'q' => { SIM_Q }
            'r' => { SIM_R }
            's' => { SIM_5 }
            't' => { SIM_T }
            'u' => { SIM_U }
            'v' => { SIM_V }
            'w' => { SIM_W }
            'x' => { SIM_X }
            'y' => { SIM_Y }
            'z' => { SIM_2 }
            '-' => { LINE }
            '_' => { BOTTOM_LINE }
            'B' => { SIM_B }
            'D' => { SIM_D }
            _ => { EMPTY }
        };
        self.send_byte(make_bin(val as u8));
        self.pos += 1;
    }

    pub fn print(&mut self, mut position: usize, s: &str){
        let mut count: u8 = 0;
        self.move_cursor(position);
        for ch in s.chars(){
            if count== 8 * DIS as u8 { break; }
            position += 2; count += 1;
            match ch {
                ch if (ch == 'b' || ch == 'B') && self.b_skin => {
                    self.print_char(position,'B');
                }
                ch if (ch == 'd' || ch == 'D') && self.d_skin => {
                    self.print_char(position, 'D');
                }
                ch if ch>='0' && ch<='9'|| ch>='a' && ch<='z' || ch == '-' || ch == '_' => {
                    self.print_char(position, ch);
                }
                ch if ch >= 'A' && ch <= 'Z' => {
                    self.print_char(position, (ch as u8 + 32) as char);
                }
                _ => { count -= 1; position -= 2; }
            }
            position %= 16*DIS;
            if position%16 == 0 { self.move_cursor(position) }
        }
    }

    /*
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
        self.stb_listen_command([0; DIS]);
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
    */
}
