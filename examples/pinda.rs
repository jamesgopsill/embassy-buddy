#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, BuddyPinda};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");

    let board = BoardBuilder::default().pinda(true).build().await;
    let pinda = board.pinda.unwrap();
    let fut = pinda_interrupt(pinda);
    fut.await;
}

async fn pinda_interrupt(sensor: BuddyPinda<'_>) -> ! {
    loop {
        if let Ok(change) = sensor.try_on_change().await {
            info!("[PINDA] {}", change);
        } else {
            info!("[PINDA] Error");
        }
    }
}
