use core::convert::Infallible;

use defmt::Format;
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

#[derive(Debug, Format)]
pub enum FilamentChanged {
    Added,
    Removed,
}

pub struct FilamentSensor<E: InputPin<Error = Infallible> + Wait<Error = Infallible>> {
    exti: E,
}

impl<E: InputPin<Error = Infallible> + Wait<Error = Infallible>> FilamentSensor<E> {
    pub fn new(exti: E) -> Self {
        Self { exti }
    }

    pub fn loaded(&mut self) -> bool {
        self.exti.is_high().unwrap()
    }

    pub async fn added(&mut self) {
        self.exti.wait_for_rising_edge().await.unwrap();
    }

    pub async fn removed(&mut self) {
        self.exti.wait_for_falling_edge().await.unwrap();
    }

    pub async fn on_change(&mut self) -> FilamentChanged {
        self.exti.wait_for_any_edge().await.unwrap();
        if self.exti.is_high().unwrap() {
            FilamentChanged::Added
        } else {
            FilamentChanged::Removed
        }
    }
}
