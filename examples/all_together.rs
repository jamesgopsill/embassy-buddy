#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_buddy::{
	buzzer::Buzzer,
	fan::Fan,
	filament_sensor::FilamentSensor,
	pinda::Pinda,
	rotary_button::RotaryButton,
	rotary_encoder::RotaryEncoder,
	thermistor::Thermistor,
	tmc2209::{Direction, TMC2209},
	Board, BuddyMutex,
};
use embassy_executor::Spawner;
use embassy_stm32::peripherals::{TIM1, TIM2};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let p = embassy_stm32::init(Default::default());
	let board = Board::default_mini(p).await;

	spawner.must_spawn(buzz(&board.buzzer));
	spawner.must_spawn(cycle(&board.fan_0, "Fan 0"));
	spawner.must_spawn(cycle(&board.fan_1, "Fan 1"));
	spawner.must_spawn(speed_camera(&board.fan_0, "Fan 0"));
	spawner.must_spawn(speed_camera(&board.fan_1, "Fan 1"));
	spawner.must_spawn(report_temp(&board.thermistors.hotend, "Hotend"));
	spawner.must_spawn(report_temp(&board.thermistors.bed, "Bed"));
	spawner.must_spawn(report_temp(&board.thermistors.board, "Board"));
	spawner.must_spawn(filament_interrupt(&board.filament_sensor));
	spawner.must_spawn(contact(&board.pinda));
	spawner.must_spawn(click(&board.rotary.button));
	spawner.must_spawn(spun(&board.rotary.encoder));
	spawner.must_spawn(step(&board.steppers.y));
	spawner.must_spawn(step(&board.steppers.x));
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

#[embassy_executor::task(pool_size = 2)]
pub async fn cycle(
	fan: &'static BuddyMutex<Fan<TIM1>>,
	label: &'static str,
) {
	{
		let mut guard = fan.lock().await;
		let fan = guard.as_mut().unwrap();
		fan.enable().await;
		info!("[{}] Enabled", label);
	}
	let mut n = 0;
	loop {
		{
			let mut guard = fan.lock().await;
			let fan = guard.as_mut().unwrap();
			fan.set_duty_cycle_fully_on().await;
			info!("[{}] On", label);
		}
		Timer::after_secs(4).await;
		{
			let mut guard = fan.lock().await;
			let fan = guard.as_mut().unwrap();
			fan.set_duty_cycle_fully_off().await;
			info!("[{}] Off", label);
		}
		Timer::after_secs(4).await;
		n += 1;
		if n > 3 {
			break;
		}
	}
	{
		let mut guard = fan.lock().await;
		let fan = guard.as_mut().unwrap();
		fan.disable().await;
		info!("[{}] Disabled", label);
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
		Timer::after_millis(250).await;
	}
}

#[embassy_executor::task(pool_size = 3)]
pub async fn report_temp(
	thermistor: &'static BuddyMutex<Thermistor>,
	label: &'static str,
) -> ! {
	loop {
		{
			let mut guard = thermistor.lock().await;
			let t = guard.as_mut().unwrap();
			let t = t.read_temperature().await;
			info!("{} T [K]: {}", label, t);
		}
		Timer::after_secs(2).await;
	}
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

#[embassy_executor::task()]
pub async fn contact(sensor: &'static BuddyMutex<Pinda>) -> ! {
	let mut guard = sensor.lock().await;
	let pinda = guard.as_mut().unwrap();
	loop {
		let change = pinda.on_change().await;
		info!("{}", change);
	}
}

#[embassy_executor::task()]
pub async fn click(sensor: &'static BuddyMutex<RotaryButton>) -> ! {
	let mut guard = sensor.lock().await;
	let btn = guard.as_mut().unwrap();
	loop {
		btn.on_click().await;
		info!("click");
	}
}

#[embassy_executor::task()]
pub async fn spun(sensor: &'static BuddyMutex<RotaryEncoder>) -> ! {
	let mut guard = sensor.lock().await;
	let encoder = guard.as_mut().unwrap();
	loop {
		let rot = encoder.wait_for_rotation().await;
		info!("{}", rot);
	}
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
			Timer::after_nanos(40).await
		}
		stepper
			.set_direction(Direction::CounterClockwise)
			.await
			.unwrap();
		for _ in 0..75 * microstep {
			stepper.toggle_step().await;
			Timer::after_nanos(40).await
		}
	}
	info!("Stepper Disabled");
	stepper.disable();
}
