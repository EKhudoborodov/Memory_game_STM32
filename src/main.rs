#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod TM1638;
mod functional;


use TM1638::LedAndKey;
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut display = LedAndKey::new(p.PB9, p.PB8, p.PB7, p.PB6);
    //println!("{}, {}", display.stb1.is_set_high(), display.stb2.is_set_high());
    let mut led = Output::new(p.PC13, Level::Low, Speed::Low);
    let mut count: u64 = 7;
    let mut pressed: u8 = 0;
    let mut difficulty: u8 = 2;
    let mut brightness: u8 = 7;
    let mut score: u64 = 0;
    let mut tmp: [u64; 2] = [0, 0];
    let mut set: [u8; 3] = [0; 3];
    let mut game_is_on: bool = false;
    let mut buttons: [u64; 10] = [0; 10];
    let mut fixed: u8 = 1;
    led.set_high();
    display.turn_on_display(brightness);
    display.clean_display();
    functional::loading(&mut display).await;
    loop {
        while game_is_on == false {
            functional::start(&mut display, brightness);
            if functional::start_menu(&mut display) {
                game_is_on = true;
                score = 0;
            }else{
                functional::start_settings(&mut display, difficulty, brightness, fixed);
                set = functional::settings(&mut display, difficulty, brightness, fixed);
                difficulty = set[0];
                brightness = set[1];
                fixed = set[2];
            }
            count += 1;
        }
        count %= 1e15 as u64;
        led.set_high();
        functional::round_start(&mut display).await;
        led.set_low();
        Timer::after(Duration::from_millis(200)).await;
        buttons = functional::show_digits(&mut display, count, difficulty, 16, fixed).await;
        count+=(3+(difficulty-1)/2) as u64;
        pressed = 0;
        while (pressed<3+(difficulty-1)/2) {
            tmp = functional::button_listen(&mut display, count);
            count = tmp[1];
            if(tmp[0] != buttons[pressed as usize]){
                functional::game_over(&mut display).await;
                game_is_on = false;
                break;
            }
            pressed += 1;
        }
        if game_is_on {
            score += 1;
            functional::right_answer(&mut display, score).await;
        }
    }
}