#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_buddy::{thermistor::Thermistor, Board, BuddyMutex};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let board = Board::init(p).await;

    spawner.must_spawn(report_temp(&board.thermistors.hotend, "Hotend"));
    spawner.must_spawn(report_temp(&board.thermistors.bed, "Bed"));
    spawner.must_spawn(report_temp(&board.thermistors.board, "Board"));
}

#[embassy_executor::task(pool_size = 3)]
pub async fn report_temp(thermistor: &'static BuddyMutex<Thermistor>, label: &'static str) -> ! {
    loop {
        {
            let mut guard = thermistor.lock().await;
            let t = guard.as_mut().unwrap();
            let t = t.last_recorded_temperature();
            info!("{} T [K]: {}", label, t);
        }
        Timer::after(Duration::from_millis(2_000)).await;
    }
}
