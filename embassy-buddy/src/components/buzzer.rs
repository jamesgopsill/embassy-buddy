use embassy_stm32::{peripherals::TIM2, timer::simple_pwm::SimplePwmChannel};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use embedded_hal::pwm::SetDutyCycle;

pub type BuddyBuzzer<'a> = Buzzer<ThreadModeRawMutex, SimplePwmChannel<'a, TIM2>>;

pub struct Buzzer<M: RawMutex, T: SetDutyCycle> {
    ch: Mutex<M, T>,
}

impl<M: RawMutex, T: SetDutyCycle> Buzzer<M, T> {
    pub fn new(ch: T) -> Self {
        Self { ch: Mutex::new(ch) }
    }

    pub async fn try_set_duty_cycle_fully_off(&self) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fully_off().unwrap();
        Ok(())
    }

    pub async fn try_set_duty_cycle_fully_on(&self) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fully_on().unwrap();
        Ok(())
    }

    pub async fn try_set_duty_cycle_fraction(
        &self,
        num: u16,
        denom: u16,
    ) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fraction(num, denom).unwrap();
        Ok(())
    }

    pub async fn try_set_duty_cycle_percent(
        &self,
        percent: u8,
    ) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_percent(percent).unwrap();
        Ok(())
    }
}
