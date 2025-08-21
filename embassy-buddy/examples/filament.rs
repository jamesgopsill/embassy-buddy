#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{BoardBuilder, BuddyFilamentSensor};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let board = BoardBuilder::default().filament_sensor(true).build().await;
    let f = board.filament_sensor.unwrap();

    let fut = filament_interrupt(f);
    fut.await;
}

async fn filament_interrupt(sensor: BuddyFilamentSensor<'_>) -> ! {
    loop {
        let change = sensor.try_on_change().await.unwrap();
        info!("Filament: {}", change);
    }
}
