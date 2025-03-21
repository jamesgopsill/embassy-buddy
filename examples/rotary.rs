#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_buddy::{
    rotary_button::RotaryButton, rotary_encoder::RotaryEncoder, Board, BuddyMutex,
};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let board = Board::init(p).await;

    spawner.must_spawn(click(&board.rotary.button));
    spawner.must_spawn(spun(&board.rotary.encoder));
}

#[embassy_executor::task()]
pub async fn click(sensor: &'static BuddyMutex<RotaryButton>) -> ! {
    let mut guard = sensor.lock().await;
    let btn = guard.as_mut().unwrap();
    loop {
        btn.on_click().await;
        info!("click");
    }
}

#[embassy_executor::task()]
pub async fn spun(sensor: &'static BuddyMutex<RotaryEncoder>) -> ! {
    let mut guard = sensor.lock().await;
    let encoder = guard.as_mut().unwrap();
    loop {
        let rot = encoder.wait_for_rotation().await;
        info!("{}", rot);
    }
}
