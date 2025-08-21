#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, BuddyBedPowerMonitor};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");

    let board = BoardBuilder::default().bed_power(true).build().await;
    let power_monitor = board.bed_power.unwrap();

    let fut = monitor(power_monitor);
    fut.await;
}

async fn monitor(sensor: BuddyBedPowerMonitor) -> ! {
    loop {
        Timer::after_secs(2).await;
        let voltage = sensor.read().await;
        info!("[BED POWER] {}", voltage);
    }
}
