use defmt::Format;
use embassy_stm32::{
    exti::{AnyChannel, ExtiInput},
    gpio::AnyPin,
};

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
    pub async fn init(
        pin_a: AnyPin,
        exti_a: AnyChannel,
        pin_b: AnyPin,
        exti_b: AnyChannel,
    ) -> RotaryEncoder {
        let input_a = ExtiInput::new(pin_a, exti_a, embassy_stm32::gpio::Pull::None);
        let input_b = ExtiInput::new(pin_b, exti_b, embassy_stm32::gpio::Pull::None);
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
