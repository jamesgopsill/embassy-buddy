#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::filament_sensor::BuddyFilamentSensor};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let f = Board::init_filament_sensor(p.PB4, p.EXTI4);

    let fut = filament_interrupt(f);
    fut.await;
}

async fn filament_interrupt(sensor: BuddyFilamentSensor<'_>) -> ! {
    loop {
        let change = sensor.try_on_change().await.unwrap();
        info!("Filament: {}", change);
    }
}
