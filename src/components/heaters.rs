#![doc = include_str!("../../docs/heaters.md")]
use embassy_stm32::{peripherals::TIM3, timer::simple_pwm::SimplePwmChannel};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use embedded_hal::pwm::SetDutyCycle;

pub type BuddyHeater<'a> = Heater<ThreadModeRawMutex, SimplePwmChannel<'a, TIM3>>;

pub struct Heater<M: RawMutex, T> {
    ch: Mutex<M, T>,
}

impl<M: RawMutex, T: SetDutyCycle> Heater<M, T> {
    pub fn new(ch: T) -> Self {
        Self { ch: Mutex::new(ch) }
    }

    pub fn try_set_duty_cycle_fully_off(&self) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fully_off().unwrap();
        Ok(())
    }

    pub async fn set_duty_cycle_fully_off(&self) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_fully_off().unwrap();
    }

    pub fn try_set_duty_cycle_fully_on(&self) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fully_on().unwrap();
        Ok(())
    }

    pub async fn set_duty_cycle_fully_on(&self) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_fully_on().unwrap();
    }

    pub async fn set_duty_cycle_fraction(&self, num: u16, denom: u16) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_fraction(num, denom).unwrap();
    }

    pub fn try_set_duty_cycle_fraction(&self, num: u16, denom: u16) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fraction(num, denom).unwrap();
        Ok(())
    }

    pub async fn set_duty_cycle_percent(&self, percent: u8) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_percent(percent).unwrap();
    }

    pub fn try_set_duty_cycle_percent(&self, percent: u8) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_percent(percent).unwrap();
        Ok(())
    }
}
