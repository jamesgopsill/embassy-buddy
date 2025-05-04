use core::convert::Infallible;

use defmt::Format;
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

pub struct RotaryEncoder<E: InputPin<Error = Infallible> + Wait<Error = Infallible>> {
    extia: E,
    extib: E,
}

#[derive(Debug, Clone, Copy, Format)]
pub enum Direction {
    CounterClockwise,
    Clockwise,
}

impl<E: InputPin<Error = Infallible> + Wait<Error = Infallible>> RotaryEncoder<E> {
    pub fn new(
        extia: E,
        extib: E,
    ) -> Self {
        Self { extia, extib }
    }

    pub async fn spun(&mut self) -> Direction {
        self.extia.wait_for_rising_edge().await.unwrap();
        if self.extib.is_high().unwrap() {
            Direction::CounterClockwise
        } else {
            Direction::Clockwise
        }
    }
}
