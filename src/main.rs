#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod functional;

use functional::Game;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rand::RngCore;
use core::arch::asm;
use cortex_m::asm::delay;
use defmt::export::display;
use defmt::println;
use embassy_executor::Spawner;
use embassy_stm32::{self, gpio::{Level, Output, Speed}, into_ref, Peripheral};
use embassy_stm32::gpio::{Flex, Input, Pin, Pull};
use embassy_stm32::gpio::Level::Low;
use embassy_stm32::peripherals::{PB7, PB8, PB9};
use embassy_time::{Duration, Timer};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut game= Game::new(p.PB9, p.PB8, p.PB7, p.PB6, p.PB4, p.PB3, p.PA12, p.PA11, p.PA10, p.PB14, p.PB15, p.PA8, p.PA9);
    //let mut display = LedAndKey::new(p.PB9, p.PB8, p.PB7, p.PB6);
    let mut led = Output::new(p.PC13, Level::Low, Speed::Low);
    let mut end:bool = false;
    let mut tmp: [u64; 17] = [0;17];
    led.set_high();
    game.loading().await;
    loop {
        game.start();
        if(!game.start_menu()){
            game.start_settings();
            game.settings();
        }else{
            while true {
                end = false;
                game.round_start().await;
                let showed = game.show_digits().await;
                while true {
                    tmp = game.button_listen();
                    if tmp[16] == 1 { end = true; break; }
                    if tmp[16] == 2 { break; }
                }
                if end { break; }
                let mut inputted: [u64; 16]= [0; 16]; let mut i: usize = 0;
                while i<16 { inputted[i] = tmp[i]; i+=1; }
                if !game.check_answer(showed, inputted).await { break; }
            }
        }
        /*
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
        functional::round_start(&mut display).await;
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
        }*/
    }
}