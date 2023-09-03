#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use display::LedAndKey;
use keyboard::Keyboard;
use embassy_stm32::{self, Peripheral};
use embassy_stm32::gpio::{AnyPin, Pin, Pull};
use embassy_stm32::time::khz;
use embassy_time::{Duration, Timer};



pub struct DisplayAndKeyboard <'d, const DIS: usize, const BUTD: usize, CLK: Pin, DIO: Pin, const ROW: usize, const COL: usize> {
    display: LedAndKey<'d, DIS, CLK, DIO>,
    keyboard: Keyboard<'d, ROW, COL>,
    is_on: [u64; BUTD]
}

impl <'d, const DIS: usize, const BUTD: usize, CLK: Pin, DIO: Pin, const ROW: usize, const COL: usize> DisplayAndKeyboard<'d, DIS, BUTD, CLK, DIO, ROW, COL>{
    pub fn new(s: [AnyPin; DIS], c:CLK, d:DIO, for_game: [u8; BUTD], inputs: [AnyPin; ROW], outputs: [AnyPin; COL]) -> DisplayAndKeyboard<'d, DIS, BUTD, CLK, DIO, ROW, COL>{
        let mut display = LedAndKey::new(s, c, d);
        let mut keyboard = Keyboard::new(inputs, outputs);
        Self { display, keyboard, is_on: [20; BUTD]}
    }

    pub fn turn_on_display(&mut self, brightness: u8){
        self.display.turn_on_display(brightness);
    }

    pub fn turn_off_display(&mut self){
        self.display.turn_off_display();
    }

    pub fn clean_display(&mut self){
        self.display.clean_display();
        self.is_on = [20; BUTD];
    }

    pub fn print(&mut self, position: usize, s: &str){
        self.display.print(position, s);
    }

    fn display_send_byte(&mut self, command: [u8; 8]){ self.display.send_byte(command); }

    pub fn swap_b_skin(&mut self){
        self.display.swap_b_skin();
    }

    pub fn swap_d_skin(&mut self){
        self.display.swap_d_skin();
    }

    pub fn print_char(&mut self, position: usize, character: char){
        self.display.print_char(position, character);
        if (character as u8) >= ('0' as u8) && (character as u8) <= ('9' as u8){self.is_on[position/2] = (character as u64) - ('0' as u64);}
        else if (character as u8) >= ('a' as u8) && (character as u8) <= ('f' as u8) { self.is_on[position/2] = (character as u64) - ('a' as u64) + 10;}
    }

    pub fn get_pressed(&mut self) -> u8 {
        let mut c: usize = 0; let mut d: usize = 0;
        let mut flag1: bool = true; let mut flag2: bool = true;
        let mut buttons: [[u8;  ROW]; COL] = [[0; ROW]; COL];
        let mut tmp1: [[u8;  ROW]; COL] = [[0; ROW]; COL];
        let mut tmp2: [[u8;  ROW]; COL] = [[0; ROW]; COL];
        let mut pressed: u8 = 0;
        while flag1 {
            buttons = self.keyboard.read_key();
            c = 0;
            for row in buttons {
                d = 0;
                for but in row {
                    if but == 1 { flag1 = false; }
                    tmp1[d][c] = but;
                    c += 1;
                }
                d += 1;
            }
        }
        while flag2 {
            buttons = self.keyboard.read_key();
            c = 0;
            flag2 = false;
            for row in buttons {
                d = 0;
                for but in row {
                    if but == 1 { flag2 = true; };
                    tmp2[d][c] = tmp1[d][c];
                    tmp1[d][c] = but;
                    c += 1;
                }
                d += 1;
            }
        }
        c = 1;
        for row in tmp2 {
            for but in row {
                if but == 1 { pressed = c as u8;break; }
                c += 1;
            }
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
    pub fn default_print(&mut self, max: u8, mut thing_for_small_random: u64) -> [u64; 18]{
        let mut position: usize = 16 ; let mut count: usize = 0; let mut tmp: usize = 0;
        let mut blinking: [u8; 16] = [0; 16];
        let mut character= '0';
        let mut f2: bool = false; let mut zero: bool = false;
        let mut pressed: u8 = 0;
        let mut res: [u64; 18] = [0; 18]; count = 0;
        self.reprint();
        loop {
            pressed = self.get_pressed();
            match pressed {
                20 => { break; }
                19 => { res[17] = 1; break; }
                1 if f2 => {
                    f2 = false;
                    self.reprint();
                    blinking = [0; 16];
                    count = tmp; position = BUTD;
                    self.cursor(blinking);
                }
                6 if !f2 => {
                    self.make_keyboard();
                    f2 = true;
                    tmp = count;
                    count = BUTD; position = 0;
                    blinking = [0; 16]; blinking[0] = 1;
                    self.cursor(blinking);
                }
                5 if position+count > BUTD && !zero => {
                    blinking[position-1] = 1;
                    if position<BUTD { blinking[position] = 0; }
                    position -= 1;
                    self.cursor(blinking);
                }
                15 if ((position<BUTD && !f2) || (position+1<BUTD && f2)) && !zero => {
                    position += 1;
                    blinking[position-1] = 0;
                    if position<16 { blinking[position] = 1; }
                    self.cursor(blinking);
                }
                p if p%5>1 && p<15 && !f2 => {
                    if zero {
                        character = (((pressed%5-2)*3+(pressed-pressed%5)/5) + ('a' as u8)) as char;
                        if (character as u8)>('g' as u8) { character = 'g'; }
                    }
                    else { character = ((pressed%5-2)*3+(pressed-pressed%5)/5 + ('1' as u8)) as char; }
                    if position<BUTD {
                        self.print_char(position*2, character);
                    } else if (count as u8) < max {
                        if !zero {
                            self.change_is_on(1);
                            self.reprint();
                        }
                        self.print_char(30, character);
                        count += 1;
                    }
                    zero = false;
                }
                10 if f2 => {
                    self.change_is_on(1);
                    self.is_on[BUTD-1] = (position+1) as u64;
                    tmp += 1;
                }
                10 if !f2 => {
                    if position == BUTD && (count as u8)<max {
                        self.change_is_on(1);
                        self.reprint();
                        zero = true;
                        self.print_char(30, '-');
                    }
                    else if position < BUTD {
                        zero = true;
                        self.print_char(2*position, '-');
                    }
                }
                11 => {
                    self.is_on = [20; BUTD];
                    if !f2 {self.reprint();}
                    count = 0; tmp = 0;
                }
                16 if count>0 => {
                    self.change_is_on(-1);
                    if !f2 { self.reprint(); }
                    count -= 1;
                }
                _ => {}
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

    fn make_keyboard(&mut self){ self.print(0,"123456789abcdefg"); }

    fn change_is_on(&mut self, edit: isize){
        let mut i: isize = 0; let m: isize = edit;
        if edit > 0 {
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
        for i in 0..BUTD {
            if self.is_on[i] < 10 {self.print_char(i*2, (self.is_on[i] as u8 + ('0' as u8)) as char);}
            else if self.is_on[i] < 17 { self.print_char(i*2, ((self.is_on[i]%10) as u8 + ('a' as u8)) as char); }
            else{ self.print_char(i*2, ' '); }
        }
    }

    pub fn cursor(&mut self, blinking: [u8; 16]){
        for i in 0..BUTD {
            if blinking[i] == 1{ self.print_char(i*2+1, 'B'); }
            else { self.print_char(i*2+1, ' '); }
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

