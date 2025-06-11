use core::convert::Infallible;

use defmt::Format;
use embassy_stm32::exti::ExtiInput;
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

pub type BuddyFilamentSensor<'a> = FilamentSensor<ThreadModeRawMutex, ExtiInput<'a>>;

#[derive(Debug, Format)]
pub enum FilamentChanged {
    Added,
    Removed,
}

pub struct FilamentSensor<M: RawMutex, T: InputPin<Error = Infallible> + Wait<Error = Infallible>> {
    exti: Mutex<M, T>,
}

impl<M: RawMutex, T: InputPin<Error = Infallible> + Wait<Error = Infallible>> FilamentSensor<M, T> {
    pub fn new(exti: T) -> Self {
        Self {
            exti: Mutex::new(exti),
        }
    }

    pub async fn available(&self) -> bool {
        let mut exti = self.exti.lock().await;
        exti.is_high().unwrap()
    }

    pub async fn added(&self) -> Result<(), TryLockError> {
        let mut exti = self.exti.try_lock()?;
        exti.wait_for_rising_edge().await.unwrap();
        Ok(())
    }

    pub async fn removed(self) -> Result<(), TryLockError> {
        let mut exti = self.exti.try_lock()?;
        exti.wait_for_falling_edge().await.unwrap();
        Ok(())
    }

    pub async fn try_on_change(&self) -> Result<FilamentChanged, TryLockError> {
        let mut exti = self.exti.try_lock()?;
        exti.wait_for_any_edge().await.unwrap();
        if exti.is_high().unwrap() {
            Ok(FilamentChanged::Added)
        } else {
            Ok(FilamentChanged::Removed)
        }
    }
}
