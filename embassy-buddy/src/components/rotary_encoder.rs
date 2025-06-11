use core::convert::Infallible;

use defmt::Format;
use embassy_stm32::exti::ExtiInput;
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

pub type BuddyRotaryEncoder<'a> = RotaryEncoder<ThreadModeRawMutex, ExtiInput<'a>>;

pub struct RotaryEncoder<M: RawMutex, T: InputPin<Error = Infallible> + Wait<Error = Infallible>> {
    extia: Mutex<M, T>,
    extib: Mutex<M, T>,
}

#[derive(Debug, Clone, Copy, Format)]
pub enum Direction {
    CounterClockwise,
    Clockwise,
}

impl<M: RawMutex, T: InputPin<Error = Infallible> + Wait<Error = Infallible>> RotaryEncoder<M, T> {
    pub fn new(
        extia: T,
        extib: T,
    ) -> Self {
        Self {
            extia: Mutex::new(extia),
            extib: Mutex::new(extib),
        }
    }

    pub async fn try_spun(&self) -> Result<Direction, TryLockError> {
        let mut extia = self.extia.try_lock()?;
        let mut extib = self.extib.try_lock()?;
        extia.wait_for_rising_edge().await.unwrap();
        if extib.is_high().unwrap() {
            Ok(Direction::CounterClockwise)
        } else {
            Ok(Direction::Clockwise)
        }
    }
}
