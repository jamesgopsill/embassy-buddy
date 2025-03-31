#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_buddy::{
	tmc2209::{Direction, TMC2209},
	Board, BuddyMutex,
};
use embassy_executor::Spawner;
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_stm32::init(Default::default());
	let board = Board::default_mini(p).await;

	spawner.must_spawn(step(&board.steppers.y));
	spawner.must_spawn(step(&board.steppers.x));
}

#[embassy_executor::task(pool_size = 2)]
pub async fn step(sensor: &'static BuddyMutex<TMC2209>) {
	let mut guard = sensor.lock().await;
	let stepper = guard.as_mut().unwrap();
	let mut n = 0;
	let microstep = 64;
	stepper.enable();
	info!("Stepper Enabled");
	loop {
		n += 1;
		if n > 5 {
			break;
		}
		stepper.set_direction(Direction::Clockwise).await.unwrap();
		for _ in 0..75 * microstep {
			stepper.toggle_step().await;
			Timer::after_nanos(50).await
		}
		stepper
			.set_direction(Direction::CounterClockwise)
			.await
			.unwrap();
		for _ in 0..75 * microstep {
			stepper.toggle_step().await;
			Timer::after_nanos(50).await
		}
	}
	info!("Stepper Disabled");
	stepper.disable();
}
