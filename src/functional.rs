#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use crate::TM1638::LedAndKey;
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

fn lights <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>){
    let mut count = 0;
    while (count < 16) {
        display.move_cursor(count*2+1);
        display.send_byte([1; 8]);
        count += 1;
    }
}
pub(crate) async fn loading <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>){
    display.clean_display();
    display.turn_on_display(7);
    display.move_cursor(0);
    display.print_char('-');
    display.send_byte([1; 8]);
    Timer::after(Duration::from_millis(100)).await;
    let mut count: u8 = 1;
    while (count<8) {
        display.move_cursor((count-1)*2);
        display.send_byte([0; 8]); display.send_byte([0; 8]);
        display.print_char('-');
        display.send_byte([1; 8]);
        Timer::after(Duration::from_millis(100)).await;
        count += 1;
    }
    display.move_cursor(14);
    display.send_byte([0; 8]); display.send_byte([0; 8]);
    display.move_cursor(16);
    display.print_char('-');
    display.send_byte([1; 8]);
    count+=1;
    Timer::after(Duration::from_millis(100)).await;
    while (count<16) {
        display.move_cursor((count-1)*2);
        display.send_byte([0; 8]); display.send_byte([0; 8]);
        display.print_char('-');
        display.send_byte([1; 8]);
        Timer::after(Duration::from_millis(100)).await;
        count += 1;
    }
}

pub(crate) fn start <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(mut display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>, bright:u8){
    display.turn_on_display(bright);
    display.clean_display();
    display.move_cursor(0);
    display.print("start");
    display.move_cursor(16);
    display.print("settings");
    lights(&mut display);
}

pub(crate) fn start_menu<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>) -> bool{
    let mut flag1: bool = true; let mut flag2: bool = true;
    let mut c: usize = 0;
    let mut pressed: u64 = 0;
    let mut buttons: [u8; 16] = [0; 16];
    let mut tmp1: [u8; 16] = [0; 16];
    let mut tmp2: [u8; 16] = [0; 16];
    while (flag1) {
        buttons = display.read_key();
        c = 0;
        for but in buttons {
            if (but==1) { flag1 = false; }
            tmp1[c] = but;
            c += 1;
        }
    }
    flag2 = true;
    while (flag2) {
        buttons = display.read_key();
        c = 0;
        flag2 = false;
        for but in buttons {
            if (but == 1) { flag2 = true; }
            tmp2[c] = tmp1[c];
            tmp1[c] = but;
            c += 1;
        }
    }
    c = 1;
    for but in tmp2 {
        if (but == 1) { pressed = c as u64; break; }
        c += 1;
    }
    return pressed < 9;
}

pub(crate) fn start_settings<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(
    mut display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>,
    difficulty: u8,
    bright: u8,
    fixed: u8,
){
    display.clean_display();
    display.move_cursor(0);
    display.print("back");
    display.move_cursor(10);
    display.print_char('b');
    display.move_cursor(14);
    display.print_char('d');
    display.move_cursor(16);
    display.print("d");
    if(difficulty<10){ display.print_char((difficulty + ('0' as u8)) as char); }
    else { display.print_char(((difficulty%10) + ('a' as u8)) as char); }
    display.move_cursor(22);
    display.print("b");
    display.print_char((bright + ('1' as u8)) as char);
    display.move_cursor(28);
    if(fixed == 1){ display.print("fy"); }
    else { display.print("fn"); }
    lights(&mut display);
}

pub(crate) fn settings<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(
    display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>,
    difficulty: u8,
    bright: u8,
    fixed: u8,
) -> [u8; 3]{
    let mut flag1: bool = true; let mut flag2: bool = true; let mut back: bool = true;
    let mut c: usize = 0;
    let mut pressed: u64 = 0;
    let mut res: [u8; 3] = [difficulty, bright, fixed];
    flag1 = true;
    while(back) {
        let mut buttons: [u8; 16] = [0; 16];
        let mut tmp1: [u8; 16] = [0; 16];
        let mut tmp2: [u8; 16] = [0; 16];
        pressed = 5;
        while (flag1) {
            buttons = display.read_key();
            c = 0;
            for but in buttons {
                if (but==1) { flag1 = false; }
                tmp1[c] = but;
                c += 1;
            }
        }
        flag2 = true;
        while (flag2) {
            buttons = display.read_key();
            c = 0;
            flag2 = false;
            for but in buttons {
                if(but == 1){ flag2 = true; };
                tmp2[c] = tmp1[c];
                tmp1[c] = but;
                c += 1;
            }
        }
        c = 1;
        for but in tmp2 {
            if (but==1) { pressed = c as u64; break; }
            c += 1;
        }
        if(pressed < 5){ back = false; }
        else if (pressed == 6){
            display.swap_b_skin();
            display.move_cursor(0);
            display.print_char('b');
            display.move_cursor(10);
            display.print_char('b');
            if(res[0] == 11){
                display.move_cursor(18);
                display.print_char('b');
            }
            display.move_cursor(22);
            display.print_char('b');
        }
        else if (pressed == 8) {
            display.swap_d_skin();
            display.move_cursor(14);
            display.print_char('d');
            display.move_cursor(16);
            display.print_char('d');
            if(res[0] == 13){
                display.move_cursor(18);
                display.print_char('d');
            }
        }
        else if (pressed == 9 || pressed == 10) {
            res[0] %= 16; res[0] += 1;
            display.move_cursor(18);
            if(res[0]<10){ display.print_char((res[0] + ('0' as u8))as char); }
            else { display.print_char(((res[0]%10) + ('a' as u8)) as char); }
        }
        else if (pressed == 12 || pressed == 13){
            res[1] += 1; res[1] %= 8;
            display.turn_on_display(res[1]);
            display.move_cursor(24);
            display.print_char((res[1] + ('1' as u8)) as char);
        }
        else if (pressed == 15 || pressed == 16){
            res[2] = 1 - res[2];
            display.move_cursor(30);
            if(res[2] == 1){ display.print_char('y'); }
            else { display.print_char('n'); }
        }
    }
    return res;
}

pub(crate) async fn round_start <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(mut display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>){
    display.clean_display();
    let mut count:u8 = 0;
    while (count<3) {
        display.move_cursor(count*6);
        display.print_char((('3' as u8) - count)as char);
        display.move_cursor(30-count*6);
        display.print_char((('3' as u8) - count)as char);
        let mut c: u8 = 0;
        while (c < 8) {
            display.move_cursor(1+c*2);
            display.send_byte([1; 8]);
            display.move_cursor(31-c*2);
            display.send_byte([1; 8]);
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
    lights(&mut display);
    Timer::after(Duration::from_millis(1000)).await;
    display.clean_display();
    Timer::after(Duration::from_millis(500)).await;
}

pub(crate) async fn show_digits<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(
    display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>,
    r: u64,
    difficulty: u8,
    max: u64,
    fixed: u8
) -> [u64; 10] {
    let mut count: u64 = 0;
    let mut res: [u64; 10] = [0; 10];
    while (count < 3+((difficulty as u64)-1)/2) {
        let mut generator = SmallRng::seed_from_u64(r+count);
        let rand_num = generator.gen_range(1..=max);
        res[count as usize] = rand_num;
        if(fixed == 1){ display.move_cursor(((rand_num - 1) * 2) as u8); }
        else {
            let mut generator = SmallRng::seed_from_u64(r+count+128);
            let rand_pos = generator.gen_range(1..=max);
            display.move_cursor(((rand_pos - 1) * 2) as u8);
        }
        if(rand_num>9){ display.print_char(((rand_num as u8) - 10 + ('a' as u8)) as char); }
        else { display.print_char(((rand_num as u8) + ('0' as u8)) as char); }
        display.send_byte([1; 8]);
        if(difficulty%2==0){ Timer::after(Duration::from_millis(500)).await; }
        else { Timer::after(Duration::from_millis(1000)).await; }
        display.clean_display();
        Timer::after(Duration::from_millis(200)).await;
        count+=1;
    }
    return res;
}

pub(crate) fn button_listen <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(
    display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>,
    mut count: u64
) -> [u64; 2]{
    let mut flag1: bool = true; let mut flag2: bool = true;
    let mut c: usize = 0;
    let mut buttons: [u8; 16] = [0; 16];
    let mut tmp1: [u8; 16] = [0; 16];
    let mut tmp2: [u8; 16] = [0; 16];
    let mut res: [u64; 2] = [0, 0];
    flag1 = true;
    while (flag1) {
        buttons = display.read_key();
        c = 0;
        for but in buttons{
            if(but==1) {
                flag1 = false;
                display.move_cursor((c as u8)*2);
                if(c<9){ display.print_char(((c+1 + ('0' as usize)) as u8) as char); }
                else{ display.print_char((((c+1)%10 + ('a' as usize)) as u8) as char) }
                display.send_byte([1; 8]);
            }
            tmp1[c] = but;
            c+=1;
        }
        count+=1;
    }
    flag2 = true;
    while (flag2) {
        buttons = display.read_key();
        c = 0;
        flag2 = false;
        for but in buttons{
            if(but==1) {
                flag2 = true;
                display.move_cursor((c as u8)*2);
                if(c<9){ display.print_char(((c+1 + ('0' as usize)) as u8) as char); }
                else{ display.print_char((((c+1)%10 + ('a' as usize)) as u8) as char) }
                display.send_byte([1; 8]);
            } else {
                display.move_cursor((c as u8)*2);
                display.send_byte([0; 8]); display.send_byte([0; 8]);
            }
            tmp2[c] = tmp1[c];
            tmp1[c] = but;
            c+=1;
        }
    }
    c = 1;
    for but in tmp2{
        if(but==1){ res[0] = c as u64; break;}
        c+=1;
    }
    res[1] = count;
    return res;
}

pub(crate) async fn right_answer<'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>, score: u64){
    display.move_cursor(0);
    display.print("SCORE");
    display.move_cursor(((15 - ((score >= 10) as u64) - ((score >= 100) as u64))*2) as u8);
    if(score>=100){ display.print_char((((score/100) as u8) + ('0' as u8)) as char); display.send_byte([0; 8]);}
    if(score>=10){ display.print_char(((((score%100)/10) as u8) + ('0' as u8)) as char); display.send_byte([0; 8]); }
    display.print_char((((score%10) as u8) + ('0' as u8)) as char);
    let mut count: u8 = 1;
    while (count < 16) {
        if (count%8 == 0){ display.move_cursor(15); }
        else { display.move_cursor((count%8)*2-1); }
        display.send_byte([0; 8]);
        if(count%8==0){ display.move_cursor(17); }
        else { display.move_cursor(31-((count-1)%8)*2); }
        display.send_byte([0; 8]);
        display.move_cursor((count%8)*2+1);
        display.send_byte([1; 8]);
        display.move_cursor(31-(count%8)*2);
        display.send_byte([1; 8]);
        Timer::after(Duration::from_millis(100)).await;
        count += 1;
    }
}

pub(crate) async fn game_over <'d, STB1: Pin, STB2: Pin, CLK: Pin, DIO: Pin>(mut display: &mut LedAndKey<'d, STB1, STB2, CLK, DIO>){
    let mut count: u8 = 0;
    display.clean_display();
    display.move_cursor(8);
    display.print("GAMEOVER");
    lights(&mut display);
    while (count<7) {
        display.turn_on_display(6-count);
        count+=1;
        Timer::after(Duration::from_millis(400)).await;
    }
    display.turn_off_display();
    Timer::after(Duration::from_millis(300)).await;
}