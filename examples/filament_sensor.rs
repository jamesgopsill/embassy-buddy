#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_buddy::{filament_sensor::FilamentSensor, Board, BuddyMutex};
use embassy_executor::Spawner;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_stm32::init(Default::default());
	let board = Board::default_mini(p).await;

	spawner.must_spawn(filament_interrupt(&board.filament_sensor));
}

#[embassy_executor::task()]
pub async fn filament_interrupt(sensor: &'static BuddyMutex<FilamentSensor>) -> ! {
	let mut guard = sensor.lock().await;
	let sensor = guard.as_mut().unwrap();
	loop {
		let change = sensor.on_change().await;
		info!("{}", change);
	}
}
