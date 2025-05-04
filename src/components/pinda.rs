use core::convert::Infallible;

use defmt::Format;
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

#[derive(Debug, Format)]
pub enum PindaStateChange {
    Contact,
    NoContact,
}

pub struct Pinda<E: InputPin<Error = Infallible> + Wait<Error = Infallible>> {
    exti: E,
}

impl<E: InputPin<Error = Infallible> + Wait<Error = Infallible>> Pinda<E> {
    pub fn new(exti: E) -> Self {
        Self { exti }
    }

    pub fn in_contact(&mut self) {
        self.exti.is_high().unwrap();
    }

    pub async fn wait_for_contact(&mut self) {
        self.exti.wait_for_rising_edge().await.unwrap();
    }

    pub async fn wait_for_no_contact(&mut self) {
        self.exti.wait_for_falling_edge().await.unwrap();
    }

    pub async fn on_change(&mut self) -> PindaStateChange {
        self.exti.wait_for_any_edge().await.unwrap();
        if self.exti.is_high().unwrap() {
            PindaStateChange::Contact
        } else {
            PindaStateChange::NoContact
        }
    }
}
