use embassy_stm32::timer::{simple_pwm::SimplePwmChannel, GeneralInstance4Channel};

pub struct HeaterConfig<T: GeneralInstance4Channel> {
	pub ch: SimplePwmChannel<'static, T>,
}

pub struct Heater<T: GeneralInstance4Channel> {
	ch: SimplePwmChannel<'static, T>,
}

impl<T: GeneralInstance4Channel> Heater<T> {
	pub fn init(mut c: HeaterConfig<T>) -> Self {
		c.ch.disable();
		Self { ch: c.ch }
	}

	pub async fn enable(&mut self) {
		self.ch.enable();
	}

	pub async fn disable(&mut self) {
		self.ch.disable();
	}

	pub async fn set_duty_cycle_fully_off(&mut self) {
		self.ch.set_duty_cycle_fully_off();
	}

	pub async fn set_duty_cycle_fully_on(&mut self) {
		self.ch.set_duty_cycle_fully_on();
	}
}
