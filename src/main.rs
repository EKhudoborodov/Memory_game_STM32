#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rand::RngCore;
use core::arch::asm;
use cortex_m::asm::delay;
use defmt::println;
use embassy_executor::Spawner;
use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{Flex, Input, Pin, Pull};
use embassy_stm32::peripherals::{PB7, PB8, PB9};
use embassy_time::{Duration, Timer};

use {defmt_rtt as _, panic_probe as _};

fn make_map() -> [bool; 304] {
    let map: [bool; 304] = [
        true, true, true, true, true, true, false, false, // 0
        false, true, true, false, false, false, false, false, // 1
        true, true, false, true, true, false, true, false, // 2
        true, true, true, true, false, false, true, false, // 3
        false, true, true, false, false, true, true, false, // 4
        true, false, true, true, false, true, true, false, // 5
        true, false, true, true, true, true, true, false, // 6
        true, true, true, false, false, false, false, false, // 7
        true, true, true, true, true, true, true, false, // 8
        true, true, true, true, false, true, true, false, // 9
        true, true, true, false, true, true, true, false, // A
        false, false, true, true, true, true, true, false, // B
        true, false, false, true, true, true, false, false, // C
        false, true, true, true, true, false, true, false, // D
        true, false, false, true, true, true, true, false, // E
        true, false, false, false, true, true, true, false, // F
        true, false, true, true, true, true, false, false, // G
        false, false, true, false, true, true, true, false, // H
        false, false, false, false, true, true, false, false, // I
        false, true, true, true, true, false, false, false, // J
        true, false, true, false, true, true, true, false, // K
        false, false, false, true, true, true, false, false, // L
        true, false, true, false, true, false, false, false, // M
        true, true, true, false, true, true, false, false, // N
        true, true, true, true, true, true, false, false, // O
        true, true, false, false, true, true, true, false, // P
        true, true, true, false, false, true, true, false, // Q
        true, true, false, false, true, true, false, false, // R
        true, false, true, true, false, true, true, false, // S
        false, false, false, true, true, true, true, false, // T
        false, true, true, true, true, true, false, false, // U
        false, false, true, true, true, false, false, false, // V
        false, true, false, true, false, true, false, false, // W
        false, true, true, false, true, true, true, false, // X
        false, true, true, true, false, true, true, false, // Y
        true, true, false, true, true, false, true, false, // Z
        false, false, false, false, false, false, true, false, // -
        false, false, false, true, false, false, false, false // _
    ];
    return map;
}

struct LedAndKey<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>{
    stb1: Output<'d, STB1>,
    stb2: Output<'d, STB2>,
    clk: Output<'d, CLK>,
    dio: Flex<'d, DIO>,
    pos: u8,
    map: [bool; 304]
}

impl <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin> LedAndKey <'d, STB1, STB2, CLK, DIO> {
    fn new(s1:STB1, s2:STB2, c:CLK, d:DIO) -> LedAndKey<'d, STB1, STB2, CLK, DIO>{
        let mut stba = Output::new(s1, Level::Low, Speed::Low);
        let mut stbb = Output::new(s2, Level::Low, Speed::Low);
        let mut clka = Output::new(c, Level::Low, Speed::Low);
        let mut dioa = Flex::new(d);
        stba.set_high();
        stbb. set_high();
        clka.set_low();
        dioa.set_as_input_output(Speed::Low, Pull::Up);
        Self { stb1: stba, stb2:stbb, clk: clka, dio: dioa, pos: 0, map: make_map() }
    }

    fn turn_on_display(&mut self, bright: u8){
        let mut count: u8 = 0;
        self.stb1.set_low();
        self.stb2.set_low();
        if(bright%2==1){self.dio.set_high()}else{self.dio.set_low();}
        self.clk.set_high();
        self.clk.set_low();
        if(bright%4>1){self.dio.set_high()}else{self.dio.set_low();}
        self.clk.set_high();
        self.clk.set_low();
        if(bright>3){self.dio.set_high();} else {self.dio.set_low();}
        self.clk.set_high();
        self.clk.set_low();
        self.dio.set_high();
        self.clk.set_high();
        self.clk.set_low();
        self.dio.set_low();
        while count<3 {
            self.clk.set_high();
            self.clk.set_low();
            count+=1;
        }
        self.dio.set_high();
        self.clk.set_high();
        self.clk.set_low();
        self.stb1.set_high();
        self.stb2.set_high();
    }

    fn turn_off_display(&mut self){
        let mut count: u8 = 0;
        self.stb1.set_low();
        self.stb2.set_low();
        self.dio.set_low();
        while (count < 7) {
            self.clk.set_high(); self.clk.set_low();
            count += 1;
        }
        self.dio.set_high();
        self.clk.set_high(); self.clk.set_low();
        self.stb1.set_high();
        self.stb2.set_high();
    }

    fn move_cursor(&mut self, mut position: u8){
        self.pos = position;
        if(position<16) {
            self.stb1.set_high();
            self.stb1.set_low();
            self.stb2.set_high();
        }else{
            self.stb2.set_high();
            self.stb2.set_low();
            self.stb1.set_high();
            position %= 16;
        }
        if(position%2 == 1){self.dio.set_high();} else {self.dio.set_low();}
        self.clk.set_high();
        self.clk.set_low();
        if(position%4>1){self.dio.set_high();} else {self.dio.set_low();}
        self.clk.set_high();
        self.clk.set_low();
        if(position%8>3){self.dio.set_high();} else {self.dio.set_low();}
        self.clk.set_high();
        self.clk.set_low();
        if(position>7){self.dio.set_high();} else {self.dio.set_low();}
        self.clk.set_high();
        self.clk.set_low();
        self.dio.set_low();
        let mut count : u8 = 0;
        while count<2 {
            self.clk.set_high();
            self.clk.set_low();
            count+=1;
        }
        self.dio.set_high();
        while count<4{
            self.clk.set_high();
            self.clk.set_low();
            count+=1;
        }
    }

    fn skip(&mut self){
        let mut count: u8 = 0;
        self.dio.set_low();
        while (count < 8) { self.clk.set_high(); self.clk.set_low(); count += 1; }
    }

    fn full_byte(&mut self){
        let mut count: u8 = 0;
        self.dio.set_high();
        while (count < 8) { self.clk.set_high(); self.clk.set_low(); count += 1; }
        self.pos += 1;
        self.pos %= 32;
    }

    fn clean_display(&mut self){
        self.stb1.set_low();
        self.stb2.set_low();
        let mut count : u8  = 0;
        self.dio.set_low();
        while count < 17{
            self.skip();
            count+=1;
        }
        self.stb1.set_high();
        self.stb2.set_high();
    }

    fn print_char(&mut self, character: char){
        let mut count: usize = 0; let mut pos: usize = 0;
        if (character == '-'){ pos = 288; }
        else if (character == '_') { pos = 296; }
        else if((character as u8)<58) { pos = (character as usize - 48) * 8; }
        else if((character as u8) < 97) { pos = 80 + 8 * (character as usize - 65);}
        else { pos = 80 + 8 * (character as usize - 97); }
        while(count<8){
            if(self.map[pos + count]){ self.dio.set_high(); } else { self.dio.set_low(); }
            self.clk.set_high(); self.clk.set_low();
            count += 1;
        }
        self.pos += 1;
    }

    fn print(&mut self, s: &str){
        let mut count: u8 = 0;
        println!("{}", self.pos);
        for ch in s.chars(){
            if(count==16){ break; }
            if ((ch >='0' && ch <= '9') || (ch>='a' && ch <= 'z') || (ch == '-') || (ch == '_') ){
                self.print_char(ch);
                self.skip();
                self.pos += 1;
                self.pos %= 32;
                count += 1;
                println!("{}", self.pos);
                if (self.pos == 0) { self.stb2.set_high(); ; self.move_cursor(0); }
                else if(self.pos == 16){ self.stb1.set_high(); self.move_cursor(16); }
            }
            else if (ch >= 'A' && ch <= 'Z') {
                let new_ch: u8 = ch as u8 + 32;
                self.print_char(new_ch as char);
                self.skip();
                self.pos += 1;
                self.pos %= 32;
                count += 1;
                if (self.pos == 0) { self.stb2.set_high(); ; self.move_cursor(0); }
                else if(self.pos == 16){ self.stb1.set_high(); self.move_cursor(16); }
            }
        }
    }

    fn read_comand(&mut self){
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

    fn read_key(&mut self) -> [bool; 16]{
        self.stb1.set_low();
        self.read_comand();
        let mut keys: [bool; 16] = [false; 16];
        let mut count: usize = 0;
        self.dio.set_high();
        while (count<32) { // 1 - 0, 2 - 8, 3 - 16, 4 - 24, 5 - 4, 6 - 12, 7 - 20, 8 - 28
            self.clk.set_low();
            if(count == 0){ keys[0] = self.dio.is_high(); }
            else if( count == 8) { keys[1] = self.dio.is_high(); }
            else if( count == 16) { keys[2] = self.dio.is_high(); }
            else if( count == 24) { keys[3] = self.dio.is_high(); }
            else if( count == 4) { keys[4] = self.dio.is_high(); }
            else if( count == 12) { keys[5] = self.dio.is_high(); }
            else if( count == 20) { keys[6] = self.dio.is_high(); }
            else if( count == 28) { keys[7] = self.dio.is_high(); }
            if(count != 31){ self.clk.set_high(); }
            count+=1;
        }
        self.stb1.set_high();
        self.stb2.set_low();
        self.read_comand();
        self.dio.set_high();
        count = 0;
        while (count<32) { // 1 - 0, 2 - 8, 3 - 16, 4 - 24, 5 - 4, 6 - 12, 7 - 20, 8 - 28
            self.clk.set_low();
            if(count == 0){ keys[8] = self.dio.is_high(); }
            else if( count == 8) { keys[9] = self.dio.is_high(); }
            else if( count == 16) { keys[10] = self.dio.is_high(); }
            else if( count == 24) { keys[11] = self.dio.is_high(); }
            else if( count == 4) { keys[12] = self.dio.is_high(); }
            else if( count == 12) { keys[13] = self.dio.is_high(); }
            else if( count == 20) { keys[14] = self.dio.is_high(); }
            else if( count == 28) { keys[15] = self.dio.is_high(); }
            if(count != 31){ self.clk.set_high(); }
            count+=1;
        }
        self.stb2.set_high();
        return keys;
    }

}

async fn loading <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(mut display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>){
    display.clean_display();
    display.turn_on_display(7);
    display.move_cursor(0);
    display.print_char('-');
    display.full_byte();
    Timer::after(Duration::from_millis(100)).await;
    let mut count: u8 = 1;
    while (count<8) {
        display.move_cursor((count-1)*2);
        display.skip(); display.skip();
        display.print_char('-');
        display.full_byte();
        Timer::after(Duration::from_millis(100)).await;
        count += 1;
    }
    display.move_cursor(14);
    display.skip(); display.skip();
    display.stb1.set_high();
    display.move_cursor(16);
    display.print_char('-');
    display.full_byte();
    count+=1;
    Timer::after(Duration::from_millis(100)).await;
    while (count<16) {
        display.move_cursor((count-1)*2);
        display.skip(); display.skip();
        display.print_char('-');
        display.full_byte();
        Timer::after(Duration::from_millis(100)).await;
        count += 1;
    }
    display.clean_display();
    count = 0;
    while (count < 16) {
        display.move_cursor(count*2+1);
        display.full_byte();
        count += 1;
    }
    display.stb2.set_high();
}

async fn round_start <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(mut display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>){
    display.clean_display();
    let mut count:u8 = 0; let mut c: u8 = 0;
    while (count<3) {
        display.move_cursor(count*6);
        display.print_char((('3' as u8) - count)as char);
        display.move_cursor(30-count*6);
        display.print_char((('3' as u8) - count)as char);
        c = 0;
        while (c < 8) {
            display.move_cursor(1+c*2);
            display.full_byte();
            display.move_cursor(31-c*2);
            display.full_byte();
            c += 1;
            Timer::after(Duration::from_millis(100)).await;
        }
        display.clean_display();
        count += 1;
    }
    display.move_cursor(14);
    display.print_char('G');
    display.move_cursor(16);
    display.print_char('O');
    count = 0;
    while (count < 16) {
        display.move_cursor(1+count*2);
        display.full_byte();
        count += 1;
    }
    Timer::after(Duration::from_millis(1000)).await;
    display.clean_display();
    Timer::after(Duration::from_millis(500)).await;
}

async fn show_digits<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(
    mut display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>,
    r: u64,
    difficulty: u8,
    max: u64
) -> [u64; 10] {
    let mut count: u64 = 0;
    let mut res: [u64; 10] = [0; 10];
    while (count < 3+((difficulty as u64)-1)/2) {
        let mut generator = SmallRng::seed_from_u64(r+count);
        let mut rand_num = generator.gen_range(1..=max);
        res[count as usize] = rand_num;
        //println!("{}: {}", (r+count), rand_num);
        display.move_cursor(((rand_num - 1) * 2) as u8);
        if(rand_num>9){ display.print_char(((rand_num as u8) - 10 + ('a' as u8)) as char); }
        else { display.print_char(((rand_num as u8) + ('0' as u8)) as char); }
        display.full_byte();
        if(difficulty%2==0){ Timer::after(Duration::from_millis(500)).await; }
        else { Timer::after(Duration::from_millis(1000)).await; }
        display.clean_display();
        Timer::after(Duration::from_millis(200)).await;
        count+=1;
    }
    return res;
}

fn button_listen <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(
    mut display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>,
    mut count: u64
) -> [u64; 2]{
    let mut flag1: bool = true; let mut flag2: bool = true;
    let mut c: usize = 0;
    let mut buttons: [bool; 16] = [false; 16];
    let mut tmp1: [bool; 16] = [false; 16];
    let mut tmp2: [bool; 16] = [false; 16];
    let mut res: [u64; 2] = [0, 0];
    flag1 = true;
    while (flag1) {
        buttons = display.read_key();
        c = 0;
        for but in buttons{
            if(but) {
                flag1 = false;
                display.move_cursor((c as u8)*2);
                if(c<9){ display.print_char(((c+1 + ('0' as usize)) as u8) as char); }
                else{ display.print_char((((c+1)%10 + ('a' as usize)) as u8) as char) }
                display.full_byte();
            }
            tmp1[c] = but;
            c+=1;
        }
        count+=1;
    }
    flag2 = true;
    display.stb1.set_high(); display.stb2.set_high();
    //println!("{}, {}", display.stb1.is_set_high(), display.stb2.is_set_high());
    while (flag2) {
        buttons = display.read_key();
        c = 0;
        flag2 = false;
        for but in buttons{
            if(but) {
                flag2 = true;
                display.move_cursor((c as u8)*2);
                if(c<9){ display.print_char(((c+1 + ('0' as usize)) as u8) as char); }
                else{ display.print_char((((c+1)%10 + ('a' as usize)) as u8) as char) }
                display.full_byte();
                display.stb1.set_high(); display.stb2.set_high();
            } else {
                display.move_cursor((c as u8)*2);
                display.skip(); display.skip();
                display.stb1.set_high(); display.stb2.set_high();
            }
            tmp2[c] = tmp1[c];
            tmp1[c] = but;
            c+=1;
        }
    }
    c = 1;
    for but in tmp2{
        if(but){ res[0] = c as u64; break;}
        c+=1;
    }
    res[1] = count;
    return res;
}

async fn right_answer<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(mut display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>, score: u64){
    display.move_cursor(0);
    display.print("SCORE");
    //println!("{}",(7 - 2 * ((score >= 10) as u64) - 2 * ((score >= 100) as u64)));
    display.move_cursor(((15 - ((score >= 10) as u64) - ((score >= 100) as u64))*2) as u8);
    if(score>=100){ display.print_char((((score/100) as u8) + ('0' as u8)) as char); display.skip();}
    if(score>=10){ display.print_char(((((score%100)/10) as u8) + ('0' as u8)) as char); display.skip(); }
    display.print_char((((score%10) as u8) + ('0' as u8)) as char);
    let mut count: u8 = 1;
    while (count < 16) {
        if (count%8 == 0){
            display.move_cursor(15);
        }
        else { display.move_cursor((count%8)*2-1); }
        display.skip();
        display.move_cursor((count%8)*2+1);
        display.full_byte();
        Timer::after(Duration::from_millis(100)).await;
        count += 1;
    }
}

async fn game_over <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(mut display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>){
    display.move_cursor(8);
    display.print("GAMEOVER");
    let mut count: u8 = 0;
    display.stb1.set_high(); display.stb2.set_high();
    while (count<8) {
        display.move_cursor(count*2+1);
        display.full_byte();
        count += 1;
    }
    count = 0;
    display.stb1.set_high(); display.stb2.set_high();
    while (count<7) {
        display.turn_on_display(6-count);
        count+=1;
        Timer::after(Duration::from_millis(400)).await;
    }
    display.turn_off_display();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut display = LedAndKey::new(p.PB9, p.PB8, p.PB7, p.PB6);
    display.dio.set_as_input_output(Speed::Low, Pull::Up);
    display.stb1.set_high(); display.stb2.set_high(); display.clk.set_low(); display.dio.set_low();
    //println!("{}, {}", display.stb1.is_set_high(), display.stb2.is_set_high());
    let mut led = Output::new(p.PC13, Level::Low, Speed::Low);
    let mut count: u64 = 0;
    let mut pressed: u8 = 0;
    let mut difficulty: u8 = 5;
    let mut score: u64 = 0;
    let mut tmp: [u64; 2] = [0, 0];
    let mut game_is_on: bool = false;
    let mut buttons: [u64; 10] = [0; 10];
    led.set_high();
    display.turn_on_display(7);
    display.clean_display();
    loop {

        if (game_is_on == false) {
            loading(&mut display).await;
            tmp = button_listen(&mut display, count);
            difficulty = tmp[0] as u8;
            game_is_on = true;
            score = 0;
            count = tmp[1];
        }
        count %= 1e15 as u64;
        led.set_high();
        round_start(&mut display).await;
        led.set_low();
        Timer::after(Duration::from_millis(200)).await;
        buttons = show_digits(&mut display, count, difficulty, 16).await;
        count+=(3+(difficulty-1)/2) as u64;
        pressed = 0;
        while (pressed<3+(difficulty-1)/2) {
            tmp = button_listen(&mut display, count);
            count = tmp[1];
            if(tmp[0] != buttons[pressed as usize]){
                game_over(&mut display).await;
                game_is_on = false;
                break;
            }
            pressed += 1;
        }
        if(game_is_on) {
            score += 1;
            right_answer(&mut display, score).await;
        }
    }
}