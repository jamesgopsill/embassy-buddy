use core::convert::Infallible;

use defmt::Format;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::Pull,
    peripherals::{EXTI8, PA8},
};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

pub type BuddyPinda<'a> = Pinda<ThreadModeRawMutex, ExtiInput<'a>>;

pub(crate) fn init_pinda<'a>(pin: PA8, ch: EXTI8) -> BuddyPinda<'a> {
    let exti = ExtiInput::new(pin, ch, Pull::None);
    Pinda::new(exti)
}

#[derive(Debug, Format)]
pub enum PindaStateChange {
    Contact,
    NoContact,
}

pub struct Pinda<M: RawMutex, T: InputPin<Error = Infallible> + Wait<Error = Infallible>> {
    exti: Mutex<M, T>,
}

impl<M: RawMutex, T: InputPin<Error = Infallible> + Wait<Error = Infallible>> Pinda<M, T> {
    pub fn new(exti: T) -> Self {
        Self {
            exti: Mutex::new(exti),
        }
    }

    pub async fn in_contact(&self) -> bool {
        let mut exti = self.exti.lock().await;
        exti.is_high().unwrap()
    }

    pub fn try_in_contact(&self) -> Result<bool, TryLockError> {
        let mut exti = self.exti.try_lock()?;
        Ok(exti.is_high().unwrap())
    }

    pub async fn try_wait_for_contact(self) -> Result<(), TryLockError> {
        let mut exti = self.exti.try_lock()?;
        exti.wait_for_rising_edge().await.unwrap();
        Ok(())
    }

    pub async fn try_wait_for_no_contact(&self) -> Result<(), TryLockError> {
        let mut exti = self.exti.try_lock()?;
        exti.wait_for_falling_edge().await.unwrap();
        Ok(())
    }

    pub async fn try_on_change(&self) -> Result<PindaStateChange, TryLockError> {
        let mut exti = self.exti.try_lock()?;
        exti.wait_for_any_edge().await.unwrap();
        if exti.is_high().unwrap() {
            Ok(PindaStateChange::Contact)
        } else {
            Ok(PindaStateChange::NoContact)
        }
    }
}
