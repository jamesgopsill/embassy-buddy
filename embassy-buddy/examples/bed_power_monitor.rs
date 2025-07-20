#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::bed_power_monitor::BuddyPowerMonitor};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    // The ADC1 needs to be initialised otherwise the bed power init will fail.
    Board::init_adc1(p.ADC1);
    let probe = Board::init_bed_power_monitor(p.PA3).await.unwrap();

    let fut = monitor(probe);
    fut.await;
}

async fn monitor(sensor: BuddyPowerMonitor<'_>) -> ! {
    loop {
        Timer::after_secs(2).await;
        let voltage = sensor.read().await;
        info!("[BED POWER] {}", voltage);
    }
}
