use defmt::Format;
use embassy_stm32::{
	exti::{AnyChannel, ExtiInput},
	gpio::{AnyPin, Pull},
};

pub struct RotaryEncoderConfig {
	pub pin_a: AnyPin,
	pub exti_a: AnyChannel,
	pub pin_b: AnyPin,
	pub exti_b: AnyChannel,
}

pub struct RotaryEncoder {
	input_a: ExtiInput<'static>,
	input_b: ExtiInput<'static>,
}

#[derive(Debug, Format, Clone, Copy)]
pub enum Direction {
	CounterClockwise,
	Clockwise,
}

impl RotaryEncoder {
	pub fn init(c: RotaryEncoderConfig) -> Self {
		let input_a = ExtiInput::new(c.pin_a, c.exti_a, Pull::None);
		let input_b = ExtiInput::new(c.pin_b, c.exti_b, Pull::None);
		Self { input_a, input_b }
	}

	pub async fn wait_for_rotation(&mut self) -> Direction {
		self.input_a.wait_for_rising_edge().await;
		if self.input_b.is_high() {
			Direction::CounterClockwise
		} else {
			Direction::Clockwise
		}
	}
}
