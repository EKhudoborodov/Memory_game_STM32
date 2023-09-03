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
use embassy_stm32::gpio::{AnyPin, Flex, Input, Pin, Pull};
use embassy_stm32::peripherals::{PB7, PB8, PB9};
use embassy_time::{Duration, Timer};

use {defmt_rtt as _, panic_probe as _};
use display_with_keyboard::DisplayAndKeyboard;

pub(crate) struct Game<'d, const DIS: usize, const BUTD: usize, CLK: Pin, DIO: Pin, const ROW: usize, const COL: usize> {
    board: DisplayAndKeyboard<'d, DIS, BUTD, CLK, DIO, ROW, COL>,
    difficulty: u8,
    brightness: u8,
    fixed: u8,
    max: u64,
    thing_for_small_random: u64,
    score: u64,
}

impl<'d, const DIS: usize, const BUTD: usize, CLK: Pin, DIO: Pin, const ROW: usize, const COL: usize> Game<'d, DIS, BUTD, CLK, DIO, ROW, COL> {
    pub(crate) fn new(s: [AnyPin; DIS], c: CLK, d: DIO, for_game: [u8; BUTD], inputs: [AnyPin; ROW], outputs: [AnyPin; COL]) -> Game<'d, DIS, BUTD, CLK, DIO, ROW, COL> {
        let b = DisplayAndKeyboard::new(s, c, d, for_game, inputs, outputs);
        Self { board: b, difficulty: 2, brightness: 4, fixed: 1, max: 16, thing_for_small_random: 0, score: 0 }
    }

    pub(crate) async fn loading(&mut self) {
        self.board.clean_display();
        self.board.turn_on_display(self.brightness);
        self.board.print_char(0, '-');
        self.board.print_char(1, 'B');
        Timer::after(Duration::from_millis(100)).await;
        for count in 0..16 {
            self.board.print_char((count - 1) * 2, ' ');
            self.board.print_char((count - 1) * 2+1, ' ');
            self.board.print_char(count*2, '-');
            self.board.print_char(count*2, 'B');
            Timer::after(Duration::from_millis(100)).await;
        }
    }

    pub(crate) fn start(&mut self) {
        self.board.turn_on_display(self.brightness);
        self.board.clean_display();
        self.board.print(0, "start");
        self.board.print(16, "settings");
    }

    pub(crate) fn start_menu(&mut self) -> bool {
        //let mut flag1: bool = true; let mut flag2: bool = true;
        let mut blinking: [u8; 16] = [1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut position: usize = 0;
        let mut i: usize = 0;
        let mut pressed: u8 = 0;
        self.board.cursor(blinking);
        loop {
            pressed = self.board.get_pressed();
            if pressed == 20 || (pressed % 5 > 1 && pressed < 15) || pressed == 10 { break; }
            if pressed == 1 { position = 0; break; }
            if pressed == 6 { position = 1; break; }
            if pressed == 5 && position == 1 {
                i = 0;
                while i < 8 { blinking[i] = 1; i += 1; }
                while i < 16 {
                    blinking[i] = 0;
                    i += 1;
                }
                position = 0;
                self.board.cursor(blinking);
            }
            else if pressed == 15 && position == 0 {
                i = 0;
                while i < 8 {
                    blinking[i] = 0;
                    i += 1;
                }
                while i < 16 {
                    blinking[i] = 1;
                    i += 1;
                }
                position = 1;
                self.board.cursor(blinking);
            }
            self.thing_for_small_random += 1;
            self.thing_for_small_random %= 1e15 as u64;
        }
        return position == 0;
    }

    pub(crate) fn start_settings(&mut self) {
        self.board.clean_display();
        self.board.print(0, "back b dd");
        if self.difficulty < 10 { self.board.print_char(18, (self.difficulty + ('0' as u8)) as char); }
        else { self.board.print_char(18, ((self.difficulty % 10) + ('a' as u8)) as char); }
        self.board.print(22, "b");
        self.board.print_char(24, (self.brightness + ('1' as u8)) as char);
        if self.fixed == 1 { self.board.print(28, "fy"); } else { self.board.print(28, "fn"); }
    }

    pub(crate) fn settings(&mut self) {
        let mut position: usize = 0;
        let mut pressed: u8 = 0;
        let mut blinking: [u8; 16] = [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        self.board.cursor(blinking);
        loop {
            pressed = self.board.get_pressed();
            match pressed {
                p if p == 19 || ((p == 20 || (p % 5 > 1 && p < 15) || p == 10) && (position == 0)) => { break; }
                5 if position>0 => {
                    position -= 1;
                    match position {
                        0 => blinking = [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        1 => blinking = [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        2 => blinking = [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                        3 => blinking = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0],
                        4 => blinking = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
                        _ => {}
                    }
                    self.board.cursor(blinking); }
                15 if position<5 =>{
                    position += 1;
                    match position {
                        1 => blinking = [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        2 => blinking = [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                        3 => blinking = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0],
                        4 => blinking = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
                        5 => blinking = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1],
                        _ => {}
                    }
                    self.board.cursor(blinking);
                }
                p if p==17 || p == 18 => {
                    match position {
                        1 => {
                            self.board.swap_b_skin();
                            self.board.print_char(0, 'b');
                            self.board.print_char(10, 'b');
                            if self.difficulty == 11 { self.board.print_char(18, 'b'); }
                            self.board.print_char(22, 'b');
                        }
                        2 => {
                            self.board.swap_d_skin();
                            self.board.print_char(14, 'd');
                            self.board.print_char(16, 'd');
                            if self.difficulty == 13 { self.board.print_char(18, 'd'); }
                        }
                        3 => {
                            if pressed == 18 {
                                self.difficulty -= 1;
                                if self.difficulty == 0 { self.difficulty = 16; }
                            } else {
                                self.difficulty %= 16;
                                self.difficulty += 1;
                            }
                            if self.difficulty < 10 { self.board.print_char(18,(self.difficulty + ('0' as u8)) as char); } else { self.board.print_char(18,((self.difficulty % 10) + ('a' as u8)) as char); }
                        }
                        4 => {
                            if pressed == 18 {
                                self.brightness -= 1;
                                if self.difficulty == 0 { self.brightness = 7; }
                            } else {
                                self.brightness += 1;
                                self.brightness %= 8;
                            }
                            self.board.turn_on_display(self.brightness);
                            self.board.print_char(24, (self.brightness + ('1' as u8)) as char);
                        }
                        5 => {
                            self.fixed = 1 - self.fixed;
                            if self.fixed == 1 { self.board.print_char(30,'y'); } else { self.board.print_char(30, 'n'); }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            self.thing_for_small_random += 1; self.thing_for_small_random %= 1e15 as u64;
        }
    }

    pub(crate) async fn round_start(&mut self) {
        self.board.clean_display();
        for count in 0..3 {
            self.board.print_char(count*6,(('3' as u8) - count as u8) as char);
            self.board.print_char(30-count*6, (('3' as u8) - count as u8) as char);
            for c in 0..8 {
                self.board.print_char(c*2+1, 'B');
                self.board.print_char(31-c*2, 'B');
                Timer::after(Duration::from_millis(100)).await;
            }
            self.board.clean_display();
        }
        self.board.print(14,"GO");
        self.lights();
        Timer::after(Duration::from_millis(1000)).await;
        self.board.clean_display();
        Timer::after(Duration::from_millis(500)).await;
    }


    pub(crate) async fn show_digits(&mut self) -> [u64; 10] {
        let mut res: [u64; 10] = [0; 10];
        let mut position = 0;
        for i in 0..(3 + ((self.difficulty as u64) - 1) / 2) {
            let mut generator = SmallRng::seed_from_u64(self.thing_for_small_random + i);
            let rand_num = generator.gen_range(1..=self.max);
            res[i as usize] = rand_num;
            if self.fixed == 1 { position = ((rand_num - 1) * 2); }
            else {
                let mut generator = SmallRng::seed_from_u64(self.thing_for_small_random + i + 128);
                position = (generator.gen_range(1..=self.max)-1)*2;
            }
            if rand_num > 9 { self.board.print_char(position as usize, ((rand_num as u8) - 10 + ('a' as u8)) as char); } else { self.board.print_char(position as usize, ((rand_num as u8) + ('0' as u8)) as char); }
            self.board.print_char(position as usize + 1, 'B');
            if self.difficulty % 2 == 0 { Timer::after(Duration::from_millis(500)).await; } else { Timer::after(Duration::from_millis(1000)).await; }
            self.board.clean_display();
            Timer::after(Duration::from_millis(200)).await;
        }
        return res;
    }

    pub(crate) fn button_listen(&mut self) -> [u64; 17] {
        let tmp = self.board.default_print(3 + (self.difficulty - 1) / 2, self.thing_for_small_random);
        let mut res: [u64; 17] = [0; 17];
        let mut i: usize = 0;
        let mut count: u8 = 0;
        if tmp[17] == 1 {
            self.quit();
            if self.quit_menu() { res[16] = 1; }
        }
        while i < 16 {
            if tmp[i] == 20 { count += 1; }
            res[i] = tmp[i];
            i += 1;
        }
        if count < 16 - (3 + (self.difficulty - 1) / 2) {
            res[16] = 2;
        }
        self.thing_for_small_random = tmp[16];
        return res;
    }

    pub(crate) async fn check_answer(&mut self, showed: [u64; 10], inputted: [u64; 16]) -> bool {
        let mut i: usize = 0;
        let mut flag: bool = true;
        while i < (3 + (self.difficulty - 1) / 2) as usize {
            if showed[i] != inputted[16 - ((3 + (self.difficulty - 1) / 2) as usize) + i] {
                self.game_over().await;
                self.score = 0;
                flag = false;
                break;
            }
            i += 1;
        }
        if flag { self.right_answer().await; }
        return flag;
    }

    async fn right_answer(&mut self) {
        self.board.clean_display();
        self.board.print(0, "SCORE");
        self.score += 1;
        if self.score >= 100 {
            self.board.print_char(26, (((self.score / 100) as u8) + ('0' as u8)) as char);
        }
        if self.score >= 10 {
            self.board.print_char(28, ((((self.score % 100) / 10) as u8) + ('0' as u8)) as char);
        }
        self.board.print_char(30, (((self.score % 10) as u8) + ('0' as u8)) as char);
        for count in  0..16 {
            self.board.print_char( (count%8)*2+1, ' ');
            self.board.print_char( 31-(count%8)*2, ' ');
            self.board.print_char( ((count+1)%8)*2+1, 'B');
            self.board.print_char( 31-((count+1)%8)*2, 'B');
            Timer::after(Duration::from_millis(100)).await;
        }
    }

    async fn game_over(&mut self) {
        self.board.clean_display();
        self.board.print(8, "GAMEOVER");
        self.lights();
        for count in 0..7 {
            self.board.turn_on_display(6 - count);
            Timer::after(Duration::from_millis(400)).await;
        }
        self.board.turn_off_display();
        Timer::after(Duration::from_millis(300)).await;
    }

    fn quit(&mut self) {
        self.board.print(1, "                ");
        self.board.print(0, "quit    yes   no");
    }

    fn quit_menu(&mut self) -> bool {
        let mut pressed: u8 = 0;
        let mut position: u8 = 1;
        let mut blinking: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1];
        self.board.cursor(blinking);
        loop {
            pressed = self.board.get_pressed();
            match pressed {
                p if p == 20 || (p % 5 > 1 && p < 15) || p == 10 => { break; }
                1 => { position = 0; break; }
                p if  p==6 || p==9 => { position = 1; break; }
                5 if position == 1 => {
                    for i in 8..11 {
                        blinking[i] = 1;
                    }
                    for i in 14..16 {
                        blinking[i] = 0;
                    }
                    position = 0;
                    self.board.cursor(blinking);
                }
                15 if position == 0 => {
                    for i in 8..11 {
                        blinking[i] = 0;
                    }
                    for i in 14..16 {
                        blinking[i] = 1;
                    }
                    position = 1;
                    self.board.cursor(blinking);
                }
                _ => {}
            }
            self.thing_for_small_random += 1; self.thing_for_small_random %= 1e15 as u64;
        }
        self.board.cursor([0; 16]);
        return position == 0;
    }

    fn lights(&mut self) {
        for count in 0..16 {
            self.board.print_char(count * 2 + 1, 'B');
        }
    }
}