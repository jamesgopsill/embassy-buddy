use embassy_stm32::{
    exti::{AnyChannel, ExtiInput},
    gpio::{AnyPin, Pull},
};

pub struct RotaryButton {
    input: ExtiInput<'static>,
}

impl RotaryButton {
    pub async fn init(pin: AnyPin, exti: AnyChannel) -> RotaryButton {
        let input = ExtiInput::new(pin, exti, Pull::None);
        RotaryButton { input }
    }

    pub async fn on_click(&mut self) {
        self.input.wait_for_rising_edge().await;
    }
}
