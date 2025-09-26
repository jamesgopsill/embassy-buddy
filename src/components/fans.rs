#![doc = include_str!("../../docs/fans.md")]
use embassy_stm32::{exti::ExtiInput, peripherals::TIM1, timer::simple_pwm::SimplePwmChannel};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use embassy_time::{Duration, Instant, WithTimeout};
use embedded_hal::pwm::SetDutyCycle;
use embedded_hal_async::digital::Wait;

pub type BuddyFan<'a> = Fan<ThreadModeRawMutex, SimplePwmChannel<'a, TIM1>, ExtiInput<'a>>;

pub struct Fan<M: RawMutex, T1, T2> {
    ch: Mutex<M, T1>,
    exti: Mutex<M, T2>,
}

impl<M: RawMutex, T1, T2> Fan<M, T1, T2> {
    pub fn new(ch: T1, exti: T2) -> Self {
        Self {
            ch: Mutex::new(ch),
            exti: Mutex::new(exti),
        }
    }
}

impl<M: RawMutex, T1: SetDutyCycle, T2> Fan<M, T1, T2> {
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

impl<M: RawMutex, T1, T2: Wait> Fan<M, T1, T2> {
    /// Calculate the current speed of the fan.
    pub async fn try_rpm(&self) -> Result<Option<f64>, TryLockError> {
        let minute_in_millis: f64 = 1_000.0 * 60.0;
        let mut exti = self.exti.try_lock()?;
        let ok = exti
            .wait_for_any_edge()
            .with_timeout(Duration::from_millis(10))
            .await
            .is_ok();
        if ok {
            let tick_one = Instant::now().as_micros() as f64;
            let ok = exti
                .wait_for_any_edge()
                .with_timeout(Duration::from_millis(10))
                .await
                .is_ok();
            if ok {
                let tick_two = Instant::now().as_micros() as f64;
                let delta = tick_two - tick_one;
                let rpm = minute_in_millis / (2.0 * delta);
                Ok(Some(rpm))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn rpm(&self) -> Option<f64> {
        let minute_in_millis: f64 = 1_000.0 * 60.0;
        let mut exti = self.exti.lock().await;
        let ok = exti
            .wait_for_any_edge()
            .with_timeout(Duration::from_millis(10))
            .await
            .is_ok();
        if ok {
            let tick_one = Instant::now().as_micros() as f64;
            let ok = exti
                .wait_for_any_edge()
                .with_timeout(Duration::from_millis(10))
                .await
                .is_ok();
            if ok {
                let tick_two = Instant::now().as_micros() as f64;
                let delta = tick_two - tick_one;
                let rpm = minute_in_millis / (2.0 * delta);
                Some(rpm)
            } else {
                None
            }
        } else {
            None
        }
    }
}
