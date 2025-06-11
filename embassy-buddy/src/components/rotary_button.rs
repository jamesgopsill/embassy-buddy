use core::convert::Infallible;

use embassy_stm32::exti::ExtiInput;
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use embedded_hal_async::digital::Wait;

pub type BuddyRotaryButton<'a> = RotaryButton<ThreadModeRawMutex, ExtiInput<'a>>;

pub struct RotaryButton<M: RawMutex, T: Wait<Error = Infallible>> {
    exti: Mutex<M, T>,
}

impl<M: RawMutex, T: Wait<Error = Infallible>> RotaryButton<M, T> {
    pub fn new(exti: T) -> Self {
        Self {
            exti: Mutex::new(exti),
        }
    }

    pub async fn try_on_click(&self) -> Result<(), TryLockError> {
        let mut exti = self.exti.try_lock()?;
        exti.wait_for_rising_edge().await.unwrap();
        Ok(())
    }
}
