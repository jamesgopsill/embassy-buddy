use defmt::Format;
use embassy_stm32::{
    exti::{AnyChannel, ExtiInput},
    gpio::AnyPin,
};

pub struct FilamentSensor {
    input: ExtiInput<'static>,
}

#[derive(Debug, Format)]
pub enum FilamentChanged {
    Added,
    Removed,
}

impl FilamentSensor {
    pub fn init(pin: AnyPin, exti: AnyChannel) -> FilamentSensor {
        let input = ExtiInput::new(pin, exti, embassy_stm32::gpio::Pull::None);
        FilamentSensor { input }
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
