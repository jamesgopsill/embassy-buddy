#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_buddy::{components::thermistor::Thermistor, Board};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Booting...");
    let p = embassy_stm32::init(Default::default());
    let adc = Board::init_thermistor_adc(p.ADC1).await;
    let f = Board::init_hotend_thermistor(adc, p.PC0);

    let fut = temp(f);
    fut.await;
}

async fn temp(mut sensor: Thermistor<'_>) -> ! {
    loop {
        Timer::after_secs(2).await;
        let temp = sensor.read_temperature().await;
        info!("Hotend Temp: {}", temp);
    }
}
