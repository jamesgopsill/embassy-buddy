#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, BuddyRotaryEncoder};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let board = BoardBuilder::default().rotary_encoder(true).build().await;
    let encoder = board.rotary_encoder.unwrap();
    let fut = spun(encoder);
    fut.await;
}

async fn spun(rotary: BuddyRotaryEncoder<'_>) -> ! {
    info!("Initialising Click");
    loop {
        let dir = rotary.try_spun().await.unwrap();
        info!("[ROTARY]: {}", dir);
    }
}
