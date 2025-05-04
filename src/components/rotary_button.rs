use core::convert::Infallible;

use embedded_hal_async::digital::Wait;

pub struct RotaryButton<E: Wait<Error = Infallible>> {
    exti: E,
}

impl<E: Wait<Error = Infallible>> RotaryButton<E> {
    pub fn new(exti: E) -> Self {
        Self { exti }
    }

    pub async fn on_click(&mut self) {
        self.exti.wait_for_rising_edge().await.unwrap();
    }
}
