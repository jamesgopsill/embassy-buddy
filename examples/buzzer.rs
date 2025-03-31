#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_buddy::{buzzer::Buzzer, Board, BuddyMutex};
use embassy_executor::Spawner;
use embassy_stm32::peripherals::TIM2;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_stm32::init(Default::default());
	let board = Board::default_mini(p).await;

	spawner.must_spawn(buzz(&board.buzzer));
}

#[embassy_executor::task()]
pub async fn buzz(buzzer: &'static BuddyMutex<Buzzer<TIM2>>) {
	let mut guard = buzzer.lock().await;
	let buzzer = guard.as_mut().unwrap();
	buzzer.enable().await;
	let mut n = 0;
	loop {
		info!("Buzz");
		buzzer.set_duty_cycle_fraction(1, 2).await;
		Timer::after_millis(100).await;
		buzzer.set_duty_cycle_fully_off().await;
		Timer::after_secs(1).await;
		n += 1;
		if n > 5 {
			break;
		}
	}
	info!("Buzz Finished");
}
