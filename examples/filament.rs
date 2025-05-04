#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{components::filament_sensor::FilamentSensor, Board};
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let f = Board::init_filament_sensor(p.PB4, p.EXTI4);

    let fut = filament_interrupt(f);
    fut.await;
}

async fn filament_interrupt(mut sensor: FilamentSensor<ExtiInput<'_>>) -> ! {
    loop {
        let change = sensor.on_change().await;
        info!("Filament: {}", change);
    }
}
