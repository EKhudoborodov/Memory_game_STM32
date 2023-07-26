#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use display::LedAndKey;
use keyboard::Keyboard;
use embassy_stm32::{self, Peripheral};
use embassy_stm32::gpio::{Pin, Pull};
use embassy_stm32::time::khz;
use embassy_time::{Duration, Timer};

pub struct DisplayAndKeyboard <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin, I1: Pin, I2: Pin, I3: Pin, I4: Pin, I5: Pin, O1: Pin, O2: Pin, O3: Pin, O4: Pin> {
    display: LedAndKey<'d, STB1, STB2, CLK, DIO>,
    keyboard: Keyboard<'d, I1, I2, I3, I4, I5, O1, O2, O3, O4>,
    is_on: [u64; 16]
}

impl <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin, I1: Pin, I2: Pin, I3: Pin, I4: Pin, I5: Pin, O1: Pin, O2: Pin, O3: Pin, O4: Pin> DisplayAndKeyboard<'d, STB1, STB2, CLK, DIO, I1, I2, I3, I4, I5, O1, O2, O3, O4>{
    pub fn new(s1:STB1, s2:STB2, c:CLK, d:DIO, i1: I1, i2: I2, i3: I3, i4: I4, i5: I5, o1: O1, o2: O2, o3: O3, o4: O4) -> DisplayAndKeyboard<'d, STB1, STB2, CLK, DIO, I1, I2, I3, I4, I5, O1, O2, O3, O4>{
        let mut display = LedAndKey::new(s1, s2, c, d);
        let mut keyboard = Keyboard::new(i1, i2, i3, i4, i5, o1, o2, o3, o4);
        Self {display: display, keyboard: keyboard, is_on: [20; 16]}
    }

    pub fn turn_on_display(&mut self, brightness: u8){
        self.display.turn_on_display(brightness);
    }

    pub fn turn_off_display(&mut self){
        self.display.turn_off_display();
    }

    pub fn clean_display(&mut self){
        self.display.clean_display();
        self.is_on = [20; 16];
    }

    pub fn print(&mut self, s: &str){
        self.display.print(s);
    }

    pub fn display_send_byte(&mut self, command: [u8; 8]){
        self.display.send_byte(command);
    }

    pub fn display_move_cursor(&mut self, position: u8){
        self.display.move_cursor(position);
    }

    pub fn swap_b_skin(&mut self){
        self.display.swap_b_skin();
    }

    pub fn swap_d_skin(&mut self){
        self.display.swap_d_skin();
    }

    pub fn print_char(&mut self, character: char, cur: usize){
        self.display.print_char(character);
        if cur != 16 {
            if (character as u8) >= ('0' as u8) && (character as u8) <= ('9' as u8){self.is_on[cur] = (character as u64) - ('0' as u64);}
            else { self.is_on[cur] = (character as u64) - ('a' as u64) + 10;}
        }
    }

    pub fn get_pressed(&mut self) -> u8 {
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
    pub fn default_print(&mut self, max: u8, mut thing_for_small_random: u64) -> [u64; 18]{
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

    pub fn cursor(&mut self, blinking: [u8; 16]){
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

