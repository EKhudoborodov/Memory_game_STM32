#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::arch::asm;
use cortex_m::asm::delay;
use defmt::export::display;
use defmt::println;
use embassy_executor::Spawner;
use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{Flex, Input, Pin, Pull};
use embassy_stm32::peripherals::{PB7, PB8, PB9};
use embassy_stm32::time::khz;
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

struct LedAndKey<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>{
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
    fn new(s1:STB1, s2:STB2, c:CLK, d:DIO) -> LedAndKey<'d, STB1, STB2, CLK, DIO>{
        let mut stba = Output::new(s1, Level::Low, Speed::Low);
        let mut stbb = Output::new(s2, Level::Low, Speed::Low);
        let mut clka = Output::new(c, Level::Low, Speed::Low);
        let mut dioa = Flex::new(d);
        stba.set_high(); stbb.set_high(); clka.set_low();
        dioa.set_as_input_output(Speed::Low, Pull::Up);
        Self { stb1: stba, stb2:stbb, clk: clka, dio: dioa, pos: 0, b_skin: false, d_skin: false, map: make_map() }
    }

    fn send_byte(&mut self, command: [u8; 8]){
        let mut i: i32 = 7;
        while i >= 0 {
            if(command[i as usize] == 1){ self.dio.set_high(); } else { self.dio.set_low(); }
            self.clk.set_high(); self.clk.set_low();
            i-=1;
        }
    }
    fn turn_on_display(&mut self, brightness: u8){
        self.stb1.set_high(); self.stb2.set_high();
        self.stb1.set_low(); self.stb2.set_low();
        self.send_byte(make_bin(brightness+136));
        self.stb1.set_high(); self.stb2.set_high();
    }

    fn turn_off_display(&mut self){
        self.stb1.set_high(); self.stb2.set_high();
        self.stb1.set_low(); self.stb2.set_low();
        self.send_byte(make_bin(128));
        self.stb1.set_high(); self.stb2.set_high();
    }

    fn move_cursor(&mut self, mut position: u8){
        self.pos = position;
        self.stb1.set_high(); self.stb2.set_high();
        if(position<16) { self.stb1.set_low(); }
        else { self.stb2.set_low(); position %= 16; }
        self.send_byte(make_bin(position+192));
    }

    fn clean_display(&mut self){
        self.stb1.set_low(); self.stb2.set_low();
        let mut count : u8  = 0;
        while count < 17{
            self.send_byte([0; 8]);
            count+=1;
        }
        self.stb1.set_high(); self.stb2.set_high();
    }

    fn swap_b_skin(&mut self){
        self.b_skin = !self.b_skin;
    }

    fn swap_d_skin(&mut self){
        self.d_skin = !self.d_skin;
    }

    fn print_char(&mut self, character: char){
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

    fn print(&mut self, s: &str){
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

    fn read_key(&mut self) -> [u8; 16]{
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

struct Keyboard <'d, I1: Pin, I2: Pin, I3: Pin, I4: Pin, I5: Pin, O1: Pin, O2: Pin, O3: Pin, O4: Pin>{
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
    fn new(i1: I1, i2: I2, i3: I3, i4: I4, i5: I5, o1: O1, o2: O2, o3: O3, o4: O4) -> Keyboard<'d ,I1, I2, I3, I4, I5, O1, O2, O3, O4>{
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
    fn read_key(&mut self) -> [u8; 20]{
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

pub(crate) struct DisplayAndKeyboard <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin, I1: Pin, I2: Pin, I3: Pin, I4: Pin, I5: Pin, O1: Pin, O2: Pin, O3: Pin, O4: Pin> {
    display: LedAndKey<'d, STB1, STB2, CLK, DIO>,
    keyboard: Keyboard<'d, I1, I2, I3, I4, I5, O1, O2, O3, O4>,
    is_on: [u64; 16]
}

impl <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin, I1: Pin, I2: Pin, I3: Pin, I4: Pin, I5: Pin, O1: Pin, O2: Pin, O3: Pin, O4: Pin> DisplayAndKeyboard<'d, STB1, STB2, CLK, DIO, I1, I2, I3, I4, I5, O1, O2, O3, O4>{
    pub(crate) fn new(s1:STB1, s2:STB2, c:CLK, d:DIO, i1: I1, i2: I2, i3: I3, i4: I4, i5: I5, o1: O1, o2: O2, o3: O3, o4: O4) -> DisplayAndKeyboard<'d, STB1, STB2, CLK, DIO, I1, I2, I3, I4, I5, O1, O2, O3, O4>{
        let mut display = LedAndKey::new(s1, s2, c, d);
        let mut keyboard = Keyboard::new(i1, i2, i3, i4, i5, o1, o2, o3, o4);
        Self {display: display, keyboard: keyboard, is_on: [20; 16]}
    }

    pub(crate) fn turn_on_display(&mut self, brightness: u8){
        self.display.turn_on_display(brightness);
    }

    pub(crate) fn turn_off_display(&mut self){
        self.display.turn_off_display();
    }

    pub(crate) fn clean_display(&mut self){
        self.display.clean_display();
        self.is_on = [20; 16];
    }

    pub(crate) fn print(&mut self, s: &str){
        self.display.print(s);
    }

    pub(crate) fn display_send_byte(&mut self, command: [u8; 8]){
        self.display.send_byte(command);
    }

    pub(crate) fn display_move_cursor(&mut self, position: u8){
        self.display.move_cursor(position);
    }

    pub(crate) fn swap_b_skin(&mut self){
        self.display.swap_b_skin();
    }

    pub(crate) fn swap_d_skin(&mut self){
        self.display.swap_d_skin();
    }

    pub(crate) fn print_char(&mut self, character: char, cur: usize){
        self.display.print_char(character);
        if(cur != 16){
            if (character as u8) >= ('0' as u8) && (character as u8) <= ('9' as u8){self.is_on[cur] = (character as u64) - ('0' as u64);}
            else { self.is_on[cur] = (character as u64) - ('a' as u64) + 10;}
        }
    }

    pub(crate) fn get_pressed(&mut self) -> u8 {
        let mut c: usize = 0;
        let mut flag1: bool = true; let mut flag2: bool = true;
        let mut buttons: [u8; 20] = [0; 20];
        let mut tmp1: [u8; 20] = [0; 20];
        let mut tmp2: [u8; 20] = [0; 20];
        let mut pressed: u8 = 0;
        while flag1 {
            buttons = self.keyboard.read_key();
            c = 0;
            for but in buttons {
                if but == 1 { flag1 = false; }
                tmp1[c] = but;
                c += 1;
            }
        }
        while flag2 {
            buttons = self.keyboard.read_key();
            c = 0;
            flag2 = false;
            for but in buttons {
                if but == 1 { flag2 = true; };
                tmp2[c] = tmp1[c];
                tmp1[c] = but;
                c += 1;
            }
        }
        c = 1;
        for but in tmp2 {
            if but == 1 { pressed = c as u8; break; }
            c += 1;
        }
        return pressed;
    }

    /*
    0 - F1, 5 - F2, 10 - #,  15 - *,
    1 - 1,  6 - 2,  11 - 3,  16 - ^,
    2 - 4,  7 - 5,  12 - 6,  17 - v,
    3 - 7,  8 - 8,  13 - 9,  18 - Esc,
    4 - <-, 9 - 0,  14 - ->, 19 - Ent,
     */
    pub(crate) fn default_print(&mut self, max: u8, mut thing_for_small_random: u64) -> [u64; 18]{
        let mut position: usize = 16 ; let mut count: usize = 0; let mut tmp: usize = 0;
        let mut blinking: [u8; 16] = [0; 16];
        let mut character= '0';
        let mut f2: bool = false; let mut zero: bool = false;
        let mut pressed: u8 = 0;
        let mut res: [u64; 18] = [0; 18]; count = 0;
        self.reprint();
        while true {
            pressed = self.get_pressed();
            if pressed == 20{ break; }
            if pressed == 19 { res[17] = 1; break; }
            if pressed == 1 && f2 {
                f2 = false;
                self.reprint();
                blinking = [0; 16];
                count = tmp; position = 16;
                self.cursor(blinking);
            }
            else if pressed == 6 && !f2 {
                self.make_keyboard();
                f2 = true;
                tmp = count;
                count = 16; position = 0;
                blinking = [0; 16]; blinking[0] = 1;
                self.cursor(blinking);
            }
            else if ((pressed == 5) && ((position+count) > 16)) && !zero {
                blinking[position-1] = 1;
                if position<16 { blinking[position] = 0; }
                position -= 1;
                self.cursor(blinking);
            }
            else if ((pressed == 15) && (((position<16) && (!f2)) || ((position<15) && (f2))) && !zero){
                position += 1;
                blinking[position-1] = 0;
                if position<16 { blinking[position] = 1; }
                self.cursor(blinking);
            }
            else if pressed%5>1 && pressed<15 && !f2{
                if zero {
                    character = (((pressed%5-2)*3+(pressed-pressed%5)/5) + ('a' as u8)) as char;
                    if (character as u8)>('g' as u8){ character = 'g'; }
                }
                else { character = ((pressed%5-2)*3+(pressed-pressed%5)/5 + ('1' as u8)) as char; }
                if position<16 {
                    self.display_move_cursor((position as u8)*2);
                    self.print_char(character, position);
                } else if ((count as u8) < max) {
                    if !zero {
                        self.change_is_on(1);
                        self.reprint();
                    }
                    self.display_move_cursor(30);
                    self.print_char(character, 15);
                    count += 1;
                }
                zero = false;
            }
            else if ((pressed%5>1 && pressed<15) || pressed == 10) && (f2){
                self.change_is_on(1);
                self.is_on[15] = (position+1) as u64;
                tmp += 1;
            }
            else if pressed == 10 && !f2{
                if position == 16 && (count as u8)<max{
                    self.change_is_on(1);
                    self.reprint();
                    self.display_move_cursor(30);
                    zero = true;
                    self.print_char('-', 16);
                }
                else if position < 16 {
                    zero = true;
                    self.display.move_cursor(2*(position as u8));
                    self.print_char('-', 16);
                }
            }
            else if pressed == 11{
                self.is_on = [20; 16];
                if !f2 {self.reprint();}
                count = 0; tmp = 0;
            }
            else if pressed == 16 && count>0{
                self.change_is_on(-1);
                if !f2 {self.reprint();}
                count -= 1;
            }
            thing_for_small_random+=1; thing_for_small_random %= 1e15 as u64;
        }
        while count<16 {
            res[count] = self.is_on[count];
            count += 1;
        }
        res[16] = thing_for_small_random;
        return res;
    }

    fn make_keyboard(&mut self){
        self.display_move_cursor(0);
        self.print("123456789abcdefg");
    }

    fn change_is_on(&mut self, edit: isize){
        let mut i: isize = 0; let m: isize = edit;
        if edit >0 {
            while (i + m) < 16 {
                if self.is_on[(i + m) as usize] < 20 { self.is_on[i as usize] = self.is_on[(i+m) as usize]; }
                i += 1;
            }
        }else{
            i = 15;
            while (i + m) >= 0 {
                if self.is_on[(i + m) as usize] < 20 { self.is_on[i as usize] = self.is_on[(i+m)as usize];  }
                else{ self.is_on[i as usize] = 20; }
                i -= 1;
            }
        }
    }

    fn reprint(&mut self){
        let mut i: usize = 0;
        while i < 16 {
            self.display_move_cursor((i*2) as u8);
            if self.is_on[i] < 10 {self.print_char((self.is_on[i] as u8 + ('0' as u8)) as char, i);}
            else if self.is_on[i] < 17 { self.print_char(((self.is_on[i]%10) as u8 + ('a' as u8)) as char, i); }
            else{ self.display_move_cursor((i*2) as u8); self.display_send_byte([0; 8]); }
            i += 1;
        }
    }

    pub(crate) fn cursor(&mut self, blinking: [u8; 16]){
        let mut i: usize = 0;
        while i<16 {
            if blinking[i] == 1{
                self.display_move_cursor((i*2+1) as u8);
                self.display_send_byte([1; 8]);
            }else{
                self.display_move_cursor((i*2+1) as u8);
                self.display_send_byte([0; 8]);
            }
            i += 1;
        }
    }

    /*async fn cursor(&mut self, blinking: [u8; 16]){
        let mut flag: bool = false;
        let mut i: usize = 0; let mut j: usize = 0;
        let mut tmp: [u8; 8] = [0; 8];
        while i<16 {
            if blinking[i] == 1 {
                self.display.move_cursor((i * 2) as u8);
                self.display.send_byte([0; 8]);
                j = 0;
                while j < 8 {
                    if self.is_on[i * 8 + j] == 1{ flag = true; };
                    j += 1;
                }
            }
            i += 1;
        }
        if flag { Timer::after(Duration::from_millis(500)).await; }
        i = 0;
        while i<16 {
            if blinking[i] == 1 {
                self.display.move_cursor((i * 2) as u8);
                j = 0;
                while j < 8 {
                    tmp[7-j] = self.is_on[i * 8 + j];
                    j += 1;
                }
                println!("{}", tmp);
                self.display.send_byte(tmp);
            }
            i+=1;
        }
        if flag { Timer::after(Duration::from_millis(500)).await; }
    }*/
}

