use defmt::Format;
use embassy_stm32::{
	exti::{AnyChannel, ExtiInput},
	gpio::{AnyPin, Pull},
};

pub struct FilamentSensorConfig {
	pub pin: AnyPin,
	pub exti: AnyChannel,
}

pub struct FilamentSensor {
	input: ExtiInput<'static>,
}

#[derive(Debug, Format)]
pub enum FilamentChanged {
	Added,
	Removed,
}

impl FilamentSensor {
	pub fn init(c: FilamentSensorConfig) -> Self {
		let input = ExtiInput::new(c.pin, c.exti, Pull::None);
		Self { input }
	}

	pub fn present(&mut self) -> bool {
		self.input.is_high()
	}

	pub async fn added(&mut self) {
		self.input.wait_for_rising_edge().await;
	}

	pub async fn removed(&mut self) {
		self.input.wait_for_falling_edge().await;
	}

	pub async fn on_change(&mut self) -> FilamentChanged {
		self.input.wait_for_any_edge().await;
		if self.input.is_high() {
			FilamentChanged::Added
		} else {
			FilamentChanged::Removed
		}
	}
}
