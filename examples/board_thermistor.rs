#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, BuddyThermistor};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let board = BoardBuilder::default()
        .board_thermistor(true, None, None, None)
        .build()
        .await;
    let probe = board.board_thermistor.unwrap();
    let fut = temp(probe);
    fut.await;
}

async fn temp(sensor: BuddyThermistor<'_>) -> ! {
    loop {
        Timer::after_secs(2).await;
        let temp = sensor.read().await;
        info!("[BED]: {}", temp);
    }
}
