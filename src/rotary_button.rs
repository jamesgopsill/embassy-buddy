use embassy_stm32::{
	exti::{AnyChannel, ExtiInput},
	gpio::{AnyPin, Pull},
};

pub struct RotaryButtonConfig {
	pub pin: AnyPin,
	pub exti: AnyChannel,
}

pub struct RotaryButton {
	input: ExtiInput<'static>,
}

impl RotaryButton {
	pub fn init(c: RotaryButtonConfig) -> Self {
		let input = ExtiInput::new(c.pin, c.exti, Pull::None);
		Self { input }
	}

	pub async fn on_click(&mut self) {
		self.input.wait_for_rising_edge().await;
	}
}
