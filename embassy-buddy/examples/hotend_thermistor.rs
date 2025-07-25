#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{Board, components::thermistors::BuddyThermistor};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let adc = Board::init_adc1(p.ADC1);
    let probe = Board::init_default_hotend_thermistor(adc, p.PC0);
    let fut = temp(probe);
    fut.await;
}

async fn temp(sensor: BuddyThermistor<'_>) -> ! {
    loop {
        Timer::after_secs(2).await;
        let temp = sensor.read().await;
        info!("Hotend Temp: {}", temp);
    }
}
