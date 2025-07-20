#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::rotary_encoder::BuddyRotaryEncoder};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let rotary = Board::init_rotary_encoder(p.PE13, p.EXTI13, p.PE15, p.EXTI15);
    let fut = spun(rotary);
    fut.await;
}

async fn spun(rotary: BuddyRotaryEncoder<'_>) -> ! {
    info!("Initialising Click");
    loop {
        let dir = rotary.try_spun().await.unwrap();
        info!("[ROTARY]: {}", dir);
    }
}
