#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_buddy::{fan::Fan, Board, BuddyMutex};
use embassy_executor::Spawner;
use embassy_stm32::peripherals::TIM1;
use embassy_time::{Duration, Timer};
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_stm32::init(Default::default());
	let board = Board::default_mini(p).await;

	spawner.must_spawn(cycle(&board.fan_0, "Fan 0"));
	spawner.must_spawn(cycle(&board.fan_1, "Fan 1"));
	spawner.must_spawn(speed_camera(&board.fan_0, "Fan 0"));
	spawner.must_spawn(speed_camera(&board.fan_1, "Fan 1"));
}

#[embassy_executor::task(pool_size = 2)]
pub async fn cycle(
	fan: &'static BuddyMutex<Fan<TIM1>>,
	label: &'static str,
) -> ! {
	loop {
		{
			let mut guard = fan.lock().await;
			let fan = guard.as_mut().unwrap();
			fan.set_duty_cycle_fully_on().await;
			info!("[{}] On", label);
		}
		Timer::after(Duration::from_millis(4_000)).await;
		{
			let mut guard = fan.lock().await;
			let fan = guard.as_mut().unwrap();
			fan.set_duty_cycle_fully_off().await;
			info!("[{}] Off", label);
		}
		Timer::after(Duration::from_millis(4_000)).await;
	}
}

#[embassy_executor::task(pool_size = 2)]
pub async fn speed_camera(
	fan: &'static BuddyMutex<Fan<TIM1>>,
	label: &'static str,
) -> ! {
	loop {
		{
			let mut guard = fan.lock().await;
			let fan = guard.as_mut().unwrap();
			if let Some(rpm) = fan.rpm().await {
				info!("[{}] RPM: {}", label, rpm);
			}
		}
	}
}
