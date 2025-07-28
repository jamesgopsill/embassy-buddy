use embassy_stm32::{
    gpio::OutputType,
    peripherals::{PA0, TIM2},
    time::khz,
    timer::simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
};
use embassy_sync::{
    blocking_mutex::raw::{RawMutex, ThreadModeRawMutex},
    mutex::{Mutex, TryLockError},
};
use embedded_hal::pwm::SetDutyCycle;

pub type BuddyBuzzer<'a> = Buzzer<ThreadModeRawMutex, SimplePwmChannel<'a, TIM2>>;

/// A convenience function to initialise the boards buzzer. Re-published on the Board struct.
pub(crate) fn init_buzzer<'a>(ch: PA0, tim: TIM2) -> BuddyBuzzer<'a> {
    let buzzer = PwmPin::new_ch1(ch, OutputType::PushPull);
    let pwm = SimplePwm::new(
        tim,
        Some(buzzer),
        None,
        None,
        None,
        khz(21),
        Default::default(),
    );
    let mut channels = pwm.split();
    channels.ch1.enable();
    Buzzer::new(channels.ch1)
}

pub struct Buzzer<M: RawMutex, T: SetDutyCycle> {
    ch: Mutex<M, T>,
}

impl<M: RawMutex, T: SetDutyCycle> Buzzer<M, T> {
    /// Creates a new instance of the buzzer.
    pub fn new(ch: T) -> Self {
        Self { ch: Mutex::new(ch) }
    }

    /// Set the duty cycle to fully off.
    pub async fn set_duty_cycle_fully_off(&self) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_fully_off().unwrap();
    }

    /// Immediately set the duty cycle to fully off or error being unable to lock the channel.
    pub fn try_set_duty_cycle_fully_off(&self) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fully_off().unwrap();
        Ok(())
    }

    /// Set the duty cycle to fully on.
    pub async fn set_duty_cycle_fully_on(&self) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_fully_on().unwrap();
    }

    /// Immediately set the duty cycle to fully on or error being unable to lock the channel.
    pub fn try_set_duty_cycle_fully_on(&self) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fully_on().unwrap();
        Ok(())
    }

    /// Set the duty cycle to fraction.
    pub async fn set_duty_cycle_fraction(&self, num: u16, denom: u16) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_fraction(num, denom).unwrap();
    }

    /// Immediately set the duty cycle fraction or error being unable to lock the channel.
    pub fn try_set_duty_cycle_fraction(&self, num: u16, denom: u16) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_fraction(num, denom).unwrap();
        Ok(())
    }

    /// Set the duty cycle to percent.
    pub async fn set_duty_cycle_percent(&self, percent: u8) {
        let mut ch = self.ch.lock().await;
        ch.set_duty_cycle_percent(percent).unwrap();
    }

    /// Immediately set the duty cycle percent or error being unable to lock the channel.
    pub fn try_set_duty_cycle_percent(&self, percent: u8) -> Result<(), TryLockError> {
        let mut ch = self.ch.try_lock()?;
        ch.set_duty_cycle_percent(percent).unwrap();
        Ok(())
    }
}
