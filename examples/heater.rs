#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_buddy::{heater::Heater, thermistor::Thermistor, Board, BuddyMutex};
use embassy_executor::Spawner;
use embassy_stm32::peripherals::TIM3;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_stm32::init(Default::default());
	let board = Board::default_mini(p).await;

	spawner.must_spawn(report_temp(&board.thermistors.bed, "Bed"));
	spawner.must_spawn(heat_and_stop(&board.heaters.bed));
}

#[embassy_executor::task()]
pub async fn report_temp(
	thermistor: &'static BuddyMutex<Thermistor>,
	label: &'static str,
) -> ! {
	loop {
		{
			let mut guard = thermistor.lock().await;
			let t = guard.as_mut().unwrap();
			let t = t.read_temperature().await;
			info!("[{}] T [K]: {}", label, t);
		}
		Timer::after_millis(500).await;
	}
}

#[embassy_executor::task()]
pub async fn heat_and_stop(heater: &'static BuddyMutex<Heater<TIM3>>) {
	let mut guard = heater.lock().await;
	let heater = guard.as_mut().unwrap();
	Timer::after_secs(5).await;
	info!("Heat On");
	heater.enable().await;
	heater.set_duty_cycle_fully_on().await;
	Timer::after_secs(20).await;
	info!("Heat Off");
	heater.set_duty_cycle_fully_off().await;
	heater.disable().await;
}
